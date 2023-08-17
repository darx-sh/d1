use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Context, Result};
use notify::event::ModifyKind;
use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

const DARX_SERVER_DIR: &str = "darx_server";
const DARX_FUNCTIONS_SUBDIR: &str = "functions";
const DARX_LIB_SUBDIR: &str = "lib";

// todo: used for mvp test only. will be removed in the future
const MVP_TEST_ENV_ID: &str = "8nvcym53y8d2";

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
  let start_time = std::time::Instant::now();
  //TODO how to config vars for dev/prod?
  let vars = vec![];
  let req = darx_core::api::dir_to_deploy_code_req(server_path, vars).await?;
  let url = format!("http://127.0.0.1:3457/deploy_code/{}", MVP_TEST_ENV_ID);
  if let Err(e) = reqwest::Client::new()
    .post(url)
    .json(&req)
    .send()
    .await?
    .error_for_status()
  {
    eprintln!("Failed to deploy code: {:?}", e);
    return Ok(());
  }
  let duration = start_time.elapsed();
  println!("Deployed code, duration: {:?}", duration.as_secs_f32());
  Ok(())
}
