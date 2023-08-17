use anyhow::Result;
use std::path::PathBuf;

pub async fn run_deploy(plugin_name: &Option<String>, dir: &str) -> Result<()> {
  let start_time = std::time::Instant::now();
  let path = PathBuf::from(dir);
  if let Some(plugin_name) = plugin_name {
    let req =
      darx_core::api::dir_to_deploy_plugin_req(plugin_name, path.as_path())
        .await?;
    let url = format!("http://127.0.0.1:3457/deploy_plugin/{}", plugin_name);
    if let Err(e) = reqwest::Client::new()
      .post(url)
      .json(&req)
      .send()
      .await?
      .error_for_status()
    {
      eprintln!("Failed to deploy plugin: {:?}", e);
      return Ok(());
    }
    let duration = start_time.elapsed();
    println!("Deployed plugin, duration: {:?}", duration.as_secs_f32());
    Ok(())
  } else {
    eprintln!("Not implemented yet");
    return Ok(());
  }
}
