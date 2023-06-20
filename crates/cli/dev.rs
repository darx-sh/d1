use anyhow::{anyhow, Context, Result};
use darx_api::{
    ApiError, Bundle, DBType, DeployFunctionsRequest, DeployFunctionsResponse,
};
use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use reqwest::Url;
use serde::Deserialize;
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
            let req = build_deploy_func_request(
                functions_path.join(output_dir).as_path(),
            )?;
            let client = reqwest::Client::new();
            match client
                .post("http://localhost:4001/app/123456/deploy_functions")
                .json(&req)
                .send()
                .await
            {
                Err(e) => println!("failed to deploy: {:?}", e),
                Ok(rsp) => {
                    let status = rsp.status();

                    if status.is_success() {
                        println!("deployed successfully");
                    } else {
                        let error = rsp.json::<serde_json::Value>().await?;
                        println!(
                            "failed to deploy. status code = {:}, body = {:}",
                            status, error
                        );
                    }
                }
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

fn build_deploy_func_request(
    output_dir: &Path,
) -> Result<DeployFunctionsRequest> {
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
        bundles.push(Bundle {
            path: entry_point.clone(),
            code,
        });
    }

    Ok(DeployFunctionsRequest {
        bundles,
        bundle_meta: meta,
        description: Some("Deployed from local dev".to_string()),
    })
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

    let req = CreateProjectRequest {
        project_id: config.project_id.clone(),
        db_type: DBType::MySQL,
        db_url: None,
    };

    match reqwest::Client::new()
        .post(format!("{}/create_project", config.url))
        .json(&req)
        .send()
        .await
    {
        Err(e) => println!("failed to create project: {:?}", e),
        Ok(rsp) => {
            let status = rsp.status();
            if status.is_success() {
                println!("project created successfully");
            } else {
                let error = rsp
                    .json::<serde_json::Value>()
                    .await
                    .with_context("failed to parse error response")?;
                println!(
                    "failed to create project. status code = {}, body = {}",
                    status, error
                );
            }
        }
    }

    todo!()
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

#[derive(Serialize)]
struct BundleReq {
    path: String,
    bytes: i64,
    checksum: String,
    checksumType: String,
}

#[derive(Deserialize)]
struct PrepareDeployRsp {
    deploymentId: String,
    bundles: Vec<BundleRsp>,
}

#[derive(Deserialize)]
struct BundleRsp {
    id: String,
    path: String,
    upload_url: String,
}
