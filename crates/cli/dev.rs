use anyhow::Result;
use std::env;
use std::path::PathBuf;

pub async fn run_dev(port: u16, dir: &str) -> Result<()> {
    let dir_path = PathBuf::from(dir);
    println!("current_dir: {}", env::current_dir()?.display());
    Ok(())
}
