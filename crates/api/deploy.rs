use std::path::{Path, PathBuf};
use tokio::fs;

use crate::{CodeReq, DeployCodeReq};
use anyhow::Result;
use async_recursion::async_recursion;
use tracing::info;

pub async fn dir_to_deploy_req(dir: &Path) -> Result<DeployCodeReq> {
    let mut file_list_path_vec = vec![];
    collect_js_file_list(&mut file_list_path_vec, dir).await?;
    let fs_path_str_vec = file_list_path_vec
        .iter()
        .map(|path| {
            path.strip_prefix(dir)
                .unwrap()
                .to_str()
                .unwrap()
                .to_string()
        })
        .collect::<Vec<_>>();
    let mut codes = vec![];

    for (path, fs_path_str) in
        file_list_path_vec.iter().zip(fs_path_str_vec.iter())
    {
        if fs_path_str.starts_with("functions/") {
            let content = fs::read_to_string(path).await?;
            codes.push(CodeReq {
                fs_path: fs_path_str.clone(),
                content,
            });
            info!("upload: {}", fs_path_str);
        } else {
            info!(
                "ignore code outside of functions directory: {}",
                fs_path_str
            );
        }
    }
    let req = DeployCodeReq {
        tag: None,
        desc: None,
        codes,
    };

    Ok(req)
}

#[async_recursion]
pub async fn collect_js_file_list(
    file_list: &mut Vec<PathBuf>,
    cur_dir: &Path,
) -> Result<()> {
    let mut entries = fs::read_dir(cur_dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        let entry_path = entry.path();
        if entry_path.is_dir() {
            collect_js_file_list(file_list, entry_path.as_path()).await?;
        } else if let Some(ext) = entry_path.extension() {
            if ext == "ts" || ext == "js" {
                file_list.push(entry_path);
            }
        }
    }
    Ok(())
}
