use anyhow::{Context, Result};
use darx_api::{Bundle, HttpRoute};
use dashmap::DashMap;
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

#[derive(Clone, Debug, Deserialize)]
pub struct DeploymentRoute {
    pub env_id: String,
    pub deploy_seq: i32,
    pub http_routes: Vec<HttpRoute>,
}

static GLOBAL_ROUTER: Lazy<DashMap<String, Vec<DeploymentRoute>>> =
    Lazy::new(|| DashMap::new());

pub async fn init_deployments(
    bundles_dir: &Path,
    pool: &sqlx::MySqlPool,
) -> Result<()> {
    let deployments = sqlx::query!(
        "\
    SELECT \
        deploys.id AS deploy_id, \
        deploys.env_id AS env_id, \
        deploys.deploy_seq AS deploy_seq, \
        http_routes.http_path AS http_path, \
        http_routes.js_entry_point AS js_entry_point, \
        http_routes.js_export AS js_export,
        http_routes.method AS method \
    FROM deploys INNER JOIN http_routes ON http_routes.deploy_id = deploys.id \
    WHERE deploys.bundle_upload_cnt = deploys.bundle_cnt"
    )
    .fetch_all(pool)
    .await
    .context("Failed to load bundles from db")?;

    for deploy in deployments.iter() {
        let http_route = HttpRoute {
            http_path: deploy.http_path.clone(),
            js_entry_point: deploy.js_entry_point.clone(),
            js_export: deploy.js_export.clone(),
            method: deploy.method.clone(),
        };

        add_single_http_route(
            deploy.env_id.as_str(),
            deploy.deploy_seq,
            http_route,
        );
    }

    let bundles = sqlx::query!(
        "SELECT \
            deploys.env_id AS env_id, \
            deploys.deploy_seq AS deploy_seq, \
            bundles.id AS id, \
            bundles.fs_path AS fs_path, \
            bundles.code AS code \
        FROM \
            bundles INNER JOIN deploys ON deploys.id = bundles.deploy_id \
        WHERE deploys.bundle_upload_cnt = deploys.bundle_cnt"
    )
    .fetch_all(pool)
    .await
    .context("Failed to load bundles from db")?;

    for bundle in bundles.iter() {
        let b = Bundle {
            id: bundle.id.clone(),
            fs_path: bundle.fs_path.clone(),
            code: bundle.code.clone(),
        };

        add_single_bundle_file(
            bundles_dir,
            bundle.env_id.as_str(),
            bundle.deploy_seq,
            &b,
        )
        .await
        .with_context( || {
            format!(
                "Failed to add bundle file on startup. env_id: {}, deploy_seq: {}",
                bundle.env_id.clone(),
                bundle.deploy_seq,
            )
        })?;
    }
    Ok(())
}

pub fn match_route(
    environment_id: &str,
    func_url: &str,
    method: &str,
) -> Option<(i32, HttpRoute)> {
    if let Some(entry) = GLOBAL_ROUTER.get(environment_id) {
        let cur_deploy = entry[0].clone();
        for route in cur_deploy.http_routes.iter() {
            if route.http_path == func_url && route.method == method {
                return Some((cur_deploy.deploy_seq, route.clone()));
            }
        }
        None
    } else {
        None
    }
}

pub fn add_route(mut route: DeploymentRoute) {
    let env_id = route.env_id.clone();
    route
        .http_routes
        .sort_by(|a, b| a.http_path.cmp(&b.http_path));
    let mut entry = GLOBAL_ROUTER.entry(env_id).or_insert_with(|| Vec::new());
    entry.insert(0, route.clone());
    entry.sort_by(|a, b| b.deploy_seq.cmp(&a.deploy_seq));
}

pub async fn add_bundle_files(
    env_id: &str,
    deploy_seq: i32,
    bundles_dir: impl AsRef<Path>,
    bundles: Vec<Bundle>,
) -> Result<()> {
    // setup bundle files
    for bundle in bundles.iter() {
        add_single_bundle_file(
            bundles_dir.as_ref(),
            env_id,
            deploy_seq,
            bundle,
        )
        .await?;
    }
    Ok(())
}

pub async fn find_bundle_dir(
    bundles_dir: impl AsRef<Path>,
    env_id: &str,
    deploy_seq: i32,
) -> Result<PathBuf> {
    let path = bundles_dir.as_ref().join(env_id).join(deploy_seq.to_string().as_str()).canonicalize().with_context(|| {
        format!(
            "Failed to canonicalize deploy directory. env_id: {}, deploy_seq: {}",
            env_id, deploy_seq
        )
    })?;
    Ok(path)
}

async fn add_single_bundle_file(
    bundles_dir: &Path,
    env_id: &str,
    deploy_seq: i32,
    bundle: &Bundle,
) -> Result<()> {
    let bundle_dir =
        setup_bundle_deployment_dir(bundles_dir.as_ref(), env_id, deploy_seq)
            .await?;
    let bundle_file = bundle_dir.join(bundle.fs_path.as_str());

    if let Some(parent) = bundle_file.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).await?;
        }
    }

    let mut file = File::create(bundle_file.as_path()).await?;
    file.write_all(bundle.code.as_ref().unwrap().as_slice())
        .await?;
    Ok(())
}

fn add_single_http_route(env_id: &str, deploy_seq: i32, route: HttpRoute) {
    let mut entry = GLOBAL_ROUTER
        .entry(env_id.to_string())
        .or_insert_with(|| Vec::new());

    let mut find_deploy = false;
    entry.iter_mut().for_each(|deploy| {
        if deploy.deploy_seq == deploy_seq {
            deploy.http_routes.push(route.clone());
            deploy
                .http_routes
                .sort_by(|a, b| a.http_path.cmp(&b.http_path));
            find_deploy = true;
        }
    });

    if !find_deploy {
        entry.push(DeploymentRoute {
            env_id: env_id.to_string(),
            deploy_seq,
            http_routes: vec![route],
        });
    }
    entry.sort_by(|a, b| b.deploy_seq.cmp(&a.deploy_seq));
}

async fn setup_bundle_deployment_dir(
    bundles_dir: &Path,
    env_id: &str,
    deploy_seq: i32,
) -> Result<PathBuf> {
    let env_dir = bundles_dir.join(env_id);
    if !env_dir.exists() {
        fs::create_dir_all(env_dir.as_path())
            .await
            .context("Failed to create env dir")?;
    }

    let deploy_dir = env_dir.join(deploy_seq.to_string().as_str());
    if !deploy_dir.exists() {
        fs::create_dir_all(deploy_dir.as_path())
            .await
            .context("Failed to create deploy dir")?;
    }
    let dir = deploy_dir.canonicalize().with_context(|| {
        format!(
            "Failed to canonicalize deploy directory. env_id: {}, deploy_seq: {}",
            env_id, deploy_seq
        )
    })?;
    Ok(dir)
}
