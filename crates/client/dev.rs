use anyhow::{anyhow, bail, Context, Result};
use darx_api::{
    ApiError, BundleMeta, BundleReq, DBType, DeployBundleReq, PrepareDeployReq,
    PrepareDeployRsp,
};
use notify::event::ModifyKind;
use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use notify_debouncer_mini::{new_debouncer, DebouncedEventKind};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

#[cfg(windows)]
const ESBUILD: &str = "esbuild.cmd";

#[cfg(not(windows))]
const ESBUILD: &str = "esbuild";

const DARX_FUNCTIONS_DIR: &str = "darx_server/functions";

// todo: used for mvp test only. will be removed in the future
const MVP_TEST_ENV_ID: &str = "cljb3ovlt0002e38vwo0xi5ge";

pub async fn run_dev(root_dir: &str) -> Result<()> {
    let dir_path = PathBuf::from(root_dir);
    let functions_path = dir_path.join(DARX_FUNCTIONS_DIR);
    fs::create_dir_all(functions_path.as_path())?;

    if let Err(error) = Command::new(ESBUILD).arg("--version").output() {
        return if error.kind() == ErrorKind::NotFound {
            Err(anyhow!("could not find esbuild"))
        } else {
            Err(anyhow!("failed to run esbuild: {:?}", error))
        };
    }

    let (tx, rx) = std::sync::mpsc::channel();

    let mut watcher = RecommendedWatcher::new(
        tx,
        Config::default().with_poll_interval(Duration::from_millis(35)),
    )?;
    watcher
        .watch(functions_path.as_path(), RecursiveMode::Recursive)
        .with_context(|| {
            format!(
                "Failed to watch functions directory: {}",
                functions_path.display()
            )
        })?;
    for event in rx.into_iter().flatten() {
        let should_update = if let EventKind::Modify(modify) = event.kind {
            matches!(modify, ModifyKind::Name(_))
                || matches!(modify, ModifyKind::Data(_))
        } else {
            false
        };

        if should_update {
            handle_file_changed(functions_path.as_path()).await?;
        }
    }

    Ok(())
}

async fn handle_file_changed(functions_path: &Path) -> Result<()> {
    let mut file_list = vec![];
    collect_js_file_list(&mut file_list, functions_path)?;
    let file_list = file_list
        .iter()
        .map(|path| {
            path.strip_prefix(functions_path).unwrap().to_str().unwrap()
        })
        .collect::<Vec<_>>();
    let output_dir = "../__output";
    let start_time = std::time::Instant::now();
    println!("Prepare files to bundle...");
    bundle_file_list(functions_path, output_dir, file_list)?;
    let (metas, bundles) =
        new_deploy_func_request(functions_path.join(output_dir).as_path())?;
    let deploy_rsp =
        prepare_deploy(MVP_TEST_ENV_ID, None, None, metas, bundles).await?;
    let mut join_set = tokio::task::JoinSet::new();
    for bundle in deploy_rsp.bundles.iter() {
        let path = functions_path
            .join(output_dir)
            .join(bundle.fs_path.as_str());
        let code = fs::read_to_string(path)?;
        join_set.spawn(upload_bundle(
            bundle.upload_url.clone(),
            bundle.fs_path.clone(),
            bundle.upload_method.clone(),
            code.clone(),
        ));
    }

    while let Some(result) = join_set.join_next().await {
        result?.with_context(|| "Failed to upload bundle")?;
    }
    let duration = start_time.elapsed();
    println!("Uploaded all files, duration: {:?}", duration.as_secs_f32());
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
    }
    Ok(())
}

fn new_deploy_func_request(
    output_dir: &Path,
) -> Result<(Vec<BundleMeta>, Vec<BundleReq>)> {
    let meta_file = output_dir.join("meta.json");
    let meta = fs::read_to_string(meta_file.as_path()).with_context(|| {
        format!("Failed to read meta file: {}", meta_file.display())
    })?;
    let meta: serde_json::Value = serde_json::from_str(&meta)?;
    let mut metas = vec![];
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
        let exports = output
            .get("exports")
            .ok_or_else(|| anyhow!("exports not found"))?
            .as_array()
            .ok_or_else(|| anyhow!("exports is not an array"))?
            .iter()
            .map(|export| {
                export
                    .as_str()
                    .ok_or_else(|| anyhow!("export is not a string"))
                    .map(|s| s.to_string())
            })
            .collect::<Result<Vec<_>>>()?;
        metas.push(BundleMeta {
            entry_point: entry_point.clone(),
            exports,
        });

        bundles.push(BundleReq {
            fs_path: entry_point.clone(),
            bytes: nbytes,
            checksum: "".to_string(),
            checksum_type: "".to_string(),
        });
    }

    Ok((metas, bundles))
}

async fn upload_bundle(
    url: String,
    fs_path: String,
    method: String,
    code: String,
) -> Result<()> {
    let req = DeployBundleReq {
        fs_path: fs_path.clone(),
        code,
    };
    let rsp = match method.as_str() {
        "POST" => reqwest::Client::new()
            .post(url.clone())
            .json(&req)
            .send()
            .await
            .with_context(|| format!("failed to upload code, url"))?,
        "PUT" => reqwest::Client::new()
            .put(url.clone())
            .json(&req)
            .send()
            .await
            .with_context(|| format!("failed to upload code, url"))?,
        _ => unimplemented!(),
    };

    if !rsp.status().is_success() {
        bail!(
            "failed to upload code, url: {:?}, method: {:?}, rsp {:?}, ",
            url,
            method,
            rsp.text().await?,
        );
    }

    // todo: query bundle upload status from control plane.
    // update bundle upload status
    // let req = json!({
    //     "status": "success"
    // });
    // let rsp = reqwest::Client::builder()
    //     .timeout(Duration::from_secs(5))
    //     .build()?
    //     .post(format!(
    //         "http://localhost:3000/api/deployments/{}/bundles/{}",
    //         deployment_id, bundle_id
    //     ))
    //     .json(&req)
    //     .send()
    //     .await
    //     .with_context(|| format!("failed to update bundle status"))?;
    println!("Uploaded file: {}", fs_path);
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
    metas: Vec<BundleMeta>,
    bundles: Vec<BundleReq>,
) -> Result<PrepareDeployRsp> {
    let req = PrepareDeployReq {
        env_id: environment_id.to_string(),
        tag,
        description,
        metas,
        bundles,
    };

    let rsp = reqwest::Client::new()
        .post(format!("http://127.0.0.1:3457/prepare_deploy"))
        .json(&req)
        .send()
        .await
        .with_context(|| "failed to send request POST to deployments")?;

    let status = rsp.status();
    if status.is_success() {
        let rsp = rsp
            .json::<PrepareDeployRsp>()
            .await
            .with_context(|| "Failed to parse response")?;
        Ok(rsp)
    } else {
        let error =
            rsp.json::<serde_json::Value>().await.with_context(|| {
                "Failed to parse error response: /api/deployments"
            })?;
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
