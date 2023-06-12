use anyhow::{anyhow, Context, Result};
use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::env;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[cfg(windows)]
const ESBUILD: &str = "esbuild.cmd";

#[cfg(not(windows))]
const ESBUILD: &str = "esbuild";

pub async fn run_dev(dir: &str) -> Result<()> {
    let dir_path = PathBuf::from(dir);
    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;
    let functions_path = dir_path.join("functions");
    if !functions_path.is_dir() {
        return Err(anyhow!("`functions` is not a directory"));
    }

    if !functions_path.exists() {
        return Err(anyhow!("`functions` directory does not exist"));
    }

    if let Err(error) = Command::new(ESBUILD).arg("--version").output() {
        return if error.kind() == ErrorKind::NotFound {
            Err(anyhow!("could not find esbuild"))
        } else {
            Err(anyhow!("failed to run esbuild: {:?}", error))
        };
    }

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
            let mut file_list = vec![];
            collect_file_list(&mut file_list, functions_path.as_path())?;
            let file_list = file_list
                .iter()
                .map(|path| {
                    path.strip_prefix(functions_path.as_path())
                        .unwrap()
                        .to_str()
                        .unwrap()
                })
                .collect::<Vec<_>>();
            bundle_file_list(functions_path.as_path(), file_list)?;
        }
    }

    Ok(())
}

fn collect_file_list(
    file_list: &mut Vec<PathBuf>,
    cur_dir: &Path,
) -> Result<()> {
    let mut entries = fs::read_dir(cur_dir)?;
    while let Some(entry) = entries.next() {
        let entry_path = entry?.path();
        if entry_path.ends_with("__output") {
            continue;
        }

        if entry_path.is_dir() {
            collect_file_list(file_list, entry_path.as_path())?;
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

fn bundle_file_list(working_dir: &Path, file_list: Vec<&str>) -> Result<()> {
    let mut command = Command::new(ESBUILD);
    command.current_dir(working_dir);
    command
        .arg("--bundle")
        .arg("--sourcemap")
        .arg("--outdir=../__output")
        .arg("--platform=browser")
        .arg("--format=esm")
        .arg("--target=esnext")
        .arg("--metafile=../__output/meta.json");
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
