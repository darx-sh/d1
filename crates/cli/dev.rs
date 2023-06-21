use anyhow::{anyhow, bail, Context, Result};
use darx_api::{
    ApiError, Bundle, DBType, DeployFunctionsRequest, DeployFunctionsResponse,
};
use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[cfg(windows)]
const ESBUILD: &str = "esbuild.cmd";

#[cfg(not(windows))]
const ESBUILD: &str = "esbuild";

pub async fn run_dev(root_dir: &str) -> Result<()> {
    let dir_path = PathBuf::from(root_dir);
    let functions_path = dir_path.join("darx_server/functions");
    fs::create_dir_all(functions_path.as_path())?;

    if let Err(error) = Command::new(ESBUILD).arg("--version").output() {
        return if error.kind() == ErrorKind::NotFound {
            Err(anyhow!("could not find esbuild"))
        } else {
            Err(anyhow!("failed to run esbuild: {:?}", error))
        };
    }

    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;

    watcher
        .watch(functions_path.as_path(), RecursiveMode::Recursive)
        .with_context(|| {
            format!(
                "Failed to watch functions directory: {}",
                functions_path.display()
            )
        })?;
    for event in rx.into_iter().flatten() {
        if let EventKind::Modify(modify) = event.kind {
            // todo: we can buffer the events and only run this once per batch.
            // or maybe try debouncer?
            let mut file_list = vec![];
            collect_js_file_list(&mut file_list, functions_path.as_path())?;
            let file_list = file_list
                .iter()
                .map(|path| {
                    path.strip_prefix(functions_path.as_path())
                        .unwrap()
                        .to_str()
                        .unwrap()
                })
                .collect::<Vec<_>>();
            let output_dir = "../__output";
            bundle_file_list(functions_path.as_path(), output_dir, file_list)?;
            let bundles = new_deploy_func_request(
                functions_path.join(output_dir).as_path(),
            )?;
            let deploy_rsp =
                prepare_deploy("123456", None, None, bundles).await?;
            for bundle in deploy_rsp.bundles.iter() {
                let path =
                    functions_path.join(output_dir).join(bundle.path.as_str());
                let code = fs::read_to_string(path)?;
                upload_bundle(
                    deploy_rsp.deploymentId.clone(),
                    bundle.id.clone(),
                    bundle.upload_url.clone(),
                    code.clone(),
                )
                .await?;
                println!("upload bundle success deploy_id: {}, bundle_id: {}, url: {}", deploy_rsp.deploymentId, bundle.id, bundle.upload_url);
            }
        }
    }

    Ok(())
}

fn collect_js_file_list(
    file_list: &mut Vec<PathBuf>,
    cur_dir: &Path,
) -> Result<()> {
    let mut entries = fs::read_dir(cur_dir)?;
    while let Some(entry) = entries.next() {
        let entry_path = entry?.path();

        if entry_path.is_dir() {
            collect_js_file_list(file_list, entry_path.as_path())?;
        } else {
            if let Some(ext) = entry_path.extension() {
                if ext == "ts" || ext == "js" {
                    file_list.push(entry_path);
                }
            }
        }
    }
    Ok(())
}

fn bundle_file_list(
    working_dir: &Path,
    output_dir: &str,
    file_list: Vec<&str>,
) -> Result<()> {
    let parent = working_dir.parent().unwrap();
    let mut command = Command::new(ESBUILD);
    command.current_dir(working_dir);
    command
        .arg("--bundle")
        .arg("--sourcemap")
        .arg(format!("--outdir={}", output_dir))
        .arg("--platform=browser")
        .arg("--format=esm")
        .arg("--target=esnext")
        .arg(format!("--metafile={}/meta.json", output_dir));
    for file in file_list {
        command.arg(file);
    }
    command.stdout(Stdio::null());
    command.stderr(Stdio::null());
    command.output()?;
    let status = command.status()?;
    if !status.success() {
        println!("esbuild finished with status: {:?}", status);
    } else {
        println!("Preparing bundle");
    }
    Ok(())
}

fn new_deploy_func_request(output_dir: &Path) -> Result<Vec<BundleReq>> {
    let meta_file = output_dir.join("meta.json");
    let meta = fs::read_to_string(meta_file.as_path()).with_context(|| {
        format!("Failed to read meta file: {}", meta_file.display())
    })?;
    let meta: serde_json::Value = serde_json::from_str(&meta)?;
    let mut bundles = vec![];
    let outputs = meta
        .get("outputs")
        .ok_or_else(|| anyhow!("No outputs found"))?
        .as_object()
        .ok_or_else(|| anyhow!("Outputs is not an object"))?;
    for (k, output) in outputs.iter() {
        // ignore .map files for now.
        if k.ends_with(".map") {
            continue;
        }

        let output = output
            .as_object()
            .ok_or_else(|| anyhow!("Output is not an object"))?;
        let nbytes = output
            .get("bytes")
            .ok_or_else(|| anyhow!("bytes not found"))?
            .as_i64()
            .ok_or_else(|| anyhow!("bytes is not a i64"))?;

        if nbytes == 0 {
            continue;
        }

        let entry_point = output
            .get("entryPoint")
            .ok_or_else(|| anyhow!("entryPoint not found"))?
            .as_str()
            .ok_or_else(|| anyhow!("entryPoint is not a string"))?
            .to_string();

        let file_path = output_dir.join(entry_point.clone());
        let code = fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read file: {}", entry_point))?;
        bundles.push(BundleReq {
            path: entry_point.clone(),
            bytes: nbytes,
            checksum: "".to_string(),
            checksumType: "".to_string(),
        });
    }

    Ok(bundles)
}

async fn upload_bundle(
    deployment_id: String,
    bundle_id: String,
    url: String,
    code: String,
) -> Result<()> {
    let rsp = reqwest::Client::new()
        .put(url)
        .body(code)
        .send()
        .await
        .with_context(|| format!("failed to upload code, url"))?;

    if !rsp.status().is_success() {
        bail!("failed to upload code, rsp {:?}", rsp);
    }

    // update bundle upload status
    let req = json!({
        "status": "success"
    });
    let rsp = reqwest::Client::new()
        .post(format!(
            "http://localhost:3000/api/deployments/{}/bundles/{}",
            deployment_id, bundle_id
        ))
        .json(&req)
        .send()
        .await
        .with_context(|| format!("failed to update bundle status"))?;
    Ok(())
}

async fn configure_project(dir: &Path) -> Result<ProjectConfig> {
    let config_file = dir.join("darx.json");
    let config = if !config_file.exists() {
        // if user login as localhost, we can create a project for them.
        // if user login as a normal user, we talk to the cloud to create a "project_id".
        // todo! cloud and localhost
        ProjectConfig {
            project_id: "123456".to_string(),
            url: "http://localhost:4001".to_string(),
        }
    } else {
        let config =
            fs::read_to_string(config_file.as_path()).with_context(|| {
                format!("Failed to read config file: {}", config_file.display())
            })?;
        let config: ProjectConfig = serde_json::from_str(&config)?;
        config
    };

    // let req = CreateProjectRequest {
    //     project_id: config.project_id.clone(),
    //     db_type: DBType::MySQL,
    //     db_url: None,
    // };
    //
    // match reqwest::Client::new()
    //     .post(format!("{}/create_project", config.url))
    //     .json(&req)
    //     .send()
    //     .await
    // {
    //     Err(e) => println!("failed to create project: {:?}", e),
    //     Ok(rsp) => {
    //         let status = rsp.status();
    //         if status.is_success() {
    //             println!("project created successfully");
    //         } else {
    //             let error = rsp
    //                 .json::<serde_json::Value>()
    //                 .await
    //                 .with_context("failed to parse error response")?;
    //             println!(
    //                 "failed to create project. status code = {}, body = {}",
    //                 status, error
    //             );
    //         }
    //     }
    // }

    todo!()
}

async fn prepare_deploy(
    environment_id: &str,
    tag: Option<String>,
    description: Option<String>,
    bundles: Vec<BundleReq>,
) -> Result<PrepareDeployRsp> {
    let req = PrepareDeployReq {
        environmentId: environment_id.to_string(),
        tag,
        description,
        bundles,
    };

    let rsp = reqwest::Client::new()
        .post(format!("http://localhost:3000/api/deployments"))
        .json(&req)
        .send()
        .await
        .with_context(|| "failed to send request POST to deployments")?;

    let status = rsp.status();
    if status.is_success() {
        let rsp = rsp
            .json::<PrepareDeployRsp>()
            .await
            .with_context(|| "failed to parse response")?;
        Ok(rsp)
    } else {
        let error = rsp
            .json::<serde_json::Value>()
            .await
            .with_context(|| "failed to parse error response")?;
        bail!(
            "failed to prepare deploy. status code = {}, body = {}",
            status,
            error
        );
    }
}

#[derive(Serialize, Deserialize)]
struct ProjectConfig {
    project_id: String,
    url: String,
}

#[derive(Serialize)]
struct PrepareDeployReq {
    environmentId: String,
    tag: Option<String>,
    description: Option<String>,
    bundles: Vec<BundleReq>,
}

#[derive(Deserialize)]
struct PrepareDeployRsp {
    deploymentId: String,
    bundles: Vec<BundleRsp>,
}

#[derive(Serialize)]
struct BundleReq {
    path: String,
    bytes: i64,
    checksum: String,
    checksumType: String,
}

#[derive(Deserialize)]
struct BundleRsp {
    id: String,
    path: String,
    upload_url: String,
}
