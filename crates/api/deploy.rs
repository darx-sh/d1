use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use tracing::info;

use crate::{Code, DeployCodeReq};

pub fn dir_to_deploy_req(dir: &Path) -> Result<DeployCodeReq> {
    let mut file_list_path_vec = vec![];
    collect_js_file_list(&mut file_list_path_vec, dir)?; //TODO should not blocking
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
            let content = fs::read_to_string(path)?; //TODO should not blocking
            codes.push(Code {
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

fn collect_js_file_list(
    file_list: &mut Vec<PathBuf>,
    cur_dir: &Path,
) -> Result<()> {
    let entries = fs::read_dir(cur_dir)?;
    for entry in entries {
        let entry_path = entry?.path();

        if entry_path.is_dir() {
            collect_js_file_list(file_list, entry_path.as_path())?;
        } else if let Some(ext) = entry_path.extension() {
            if ext == "ts" || ext == "js" {
                file_list.push(entry_path);
            }
        }
    }
    Ok(())
}
