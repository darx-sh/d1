use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Context, Result};
use notify::event::ModifyKind;
use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

use darx_api::{Code, DeployCodeReq};

const DARX_SERVER_DIR: &str = "darx_server";
const DARX_FUNCTIONS_SUBDIR: &str = "functions";
const DARX_LIB_SUBDIR: &str = "lib";

// todo: used for mvp test only. will be removed in the future
const MVP_TEST_ENV_ID: &str = "cljb3ovlt0002e38vwo0xi5ge";

pub async fn run_dev(root_dir: &str) -> Result<()> {
    let root_path = PathBuf::from(root_dir);
    let server_path = root_path.join(DARX_SERVER_DIR);
    let functions_path = server_path.join(DARX_FUNCTIONS_SUBDIR);
    let libs_path = server_path.join(DARX_LIB_SUBDIR);
    fs::create_dir_all(functions_path.as_path()).with_context(|| {
        format!(
            "Failed to create `functions` directory: {:?}",
            functions_path.display()
        )
    })?;
    fs::create_dir_all(libs_path.as_path()).with_context(|| {
        format!(
            "Failed to create `libs` directory: {:?}",
            libs_path.display()
        )
    })?;

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
            handle_file_changed(server_path.as_path()).await?;
        }
    }

    Ok(())
}

async fn handle_file_changed(server_path: &Path) -> Result<()> {
    let mut file_list_path_vec = vec![];
    collect_js_file_list(&mut file_list_path_vec, server_path)?;
    let fs_path_str_vec = file_list_path_vec
        .iter()
        .map(|path| {
            path.strip_prefix(server_path)
                .unwrap()
                .to_str()
                .unwrap()
                .to_string()
        })
        .collect::<Vec<_>>();
    let start_time = std::time::Instant::now();
    let mut codes = vec![];

    for (path, fs_path_str) in
        file_list_path_vec.iter().zip(fs_path_str_vec.iter())
    {
        let content = fs::read_to_string(path)?;
        codes.push(Code {
            fs_path: fs_path_str.clone(),
            content,
        });
    }
    let req = DeployCodeReq {
        tag: None,
        desc: None,
        codes,
    };
    let url = format!("http://127.0.0.1:3457/deploy_code/{}", MVP_TEST_ENV_ID);
    match reqwest::Client::new()
        .post(url)
        .json(&req)
        .send()
        .await?
        .error_for_status()
    {
        Err(e) => {
            eprintln!("Failed to deploy code: {:?}", e);
            return Ok(());
        }
        Ok(_) => {}
    }
    let duration = start_time.elapsed();
    println!("Deployed code, duration: {:?}", duration.as_secs_f32());
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
