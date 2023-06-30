use anyhow::{Context, Result};
use dashmap::DashMap;

use futures_util::StreamExt as _;
use once_cell::sync::Lazy;
use redis::AsyncCommands;
use s3::creds::Credentials;
use s3::{Bucket, Region};
use serde::Deserialize;
use sqlx::MySqlPool;
use std::env;
use std::path::Path;
use tokio::fs;
use tokio::task::JoinSet;

pub async fn start_cmd_handler() -> Result<()> {
    let redis_client = redis::Client::open(
        env::var("REDIS_URL").expect("REDIS_URL should be configured"),
    )?;
    // todo: handle the case where the connection is closed by redis server.
    let mut pubsub = redis_client
        .get_async_connection()
        .await
        .with_context(|| format!("Failed to connect to redis"))?
        .into_pubsub();
    pubsub.subscribe("deploy").await?;
    let mut pubsub_stream = pubsub.on_message();
    while let Some(msg) = pubsub_stream.next().await {
        let payload: String = msg.get_payload()?;
        let deployment = serde_json::from_str::<Deployment>(&payload).unwrap();
        deploy_bundles(deployment.clone()).await.with_context(|| {
            format!("Failed to deploy bundles: {:?}", deployment)
        })?;
    }

    panic!("no more messages");

    Ok(())
}

pub async fn init_global_router() -> Result<()> {
    let pool = MySqlPool::connect(
        &env::var("DATABASE_URL").expect("DATABASE_URL should be configured"),
    )
    .await?;
    // todo: use fetch to and stream to avoid loading all records into memory.
    let records = sqlx::query!(
        r#"
SELECT 
    environmentId AS environment_id, 
    deploySeq AS deploy_seq, 
    Bundle.id AS bundle_id,
    Bundle.fsPath AS bundle_fs_path,
    HttpRoute.id AS http_route_id,
    HttpRoute.httpPath AS http_path,
    HttpRoute.method AS http_method,
    HttpRoute.jsEntryPoint AS js_entry_point,
    HttpRoute.jsExport AS js_export
FROM 
    Deployment
INNER JOIN Bundle ON Bundle.deploymentId = Deployment.id
LEFT JOIN HttpRoute ON HttpRoute.deploymentId = Deployment.id
WHERE Deployment.bundleUploadCnt = Deployment.bundleCnt
"#
    )
    .fetch_all(&pool)
    .await?;

    for r in records.iter() {
        let env_id = r.environment_id.clone();
        let deploy_seq = r.deploy_seq as i64;
        let bundle = Bundle {
            id: r.bundle_id.clone(),
            fs_path: r.bundle_fs_path.clone(),
        };
        let http_route = HttpRoute {
            id: r.http_route_id.as_ref().unwrap().clone(),
            http_path: r.http_path.as_ref().unwrap().clone(),
            method: r.http_method.as_ref().unwrap().clone(),
            js_entry_point: r.js_entry_point.as_ref().unwrap().clone(),
            js_export: r.js_export.as_ref().unwrap().clone(),
        };

        let mut entry = GLOBAL_ROUTER
            .entry(env_id.clone())
            .or_insert_with(|| Vec::new());
        let mut found_deploy_seq = false;
        for deploy in entry.iter_mut() {
            if deploy.deploy_seq == deploy_seq {
                let mut found_bundle = false;
                for bundle in deploy.bundles.iter_mut() {
                    if bundle.id == r.bundle_id {
                        found_bundle = true;
                        break;
                    }
                }

                if !found_bundle {
                    deploy.bundles.push(bundle.clone());
                }

                let mut found_route = false;
                for route in deploy.http_routes.iter_mut() {
                    if route.id == http_route.id {
                        found_route = true;
                        break;
                    }
                }
                if !found_route {
                    deploy.http_routes.push(http_route.clone());
                }
                deploy
                    .http_routes
                    .sort_by(|a, b| a.http_path.cmp(&b.http_path));
                found_deploy_seq = true;
                break;
            }
        }
        if !found_deploy_seq {
            entry.push(Deployment {
                environment_id: env_id.clone(),
                deploy_seq,
                bundles: vec![bundle.clone()],
                http_routes: vec![http_route.clone()],
            });
            entry.sort_by(|a, b| b.deploy_seq.cmp(&a.deploy_seq));
        }
    }
    Ok(())
}

pub async fn download_bundles() -> Result<()> {
    let mut join_set = JoinSet::new();
    for deploys in GLOBAL_ROUTER.iter() {
        for deploy in deploys.iter() {
            join_set.spawn(deploy_bundles(deploy.clone()));
        }
    }
    while let Some(result) = join_set.join_next().await {
        result?.with_context(|| "Failed to finish bundle download")?;
    }
    Ok(())
}

#[derive(Clone, Debug, Deserialize)]
struct Deployment {
    environment_id: String,
    deploy_seq: i64,
    bundles: Vec<Bundle>,
    http_routes: Vec<HttpRoute>,
}

#[derive(Clone, Debug, Deserialize)]
struct Bundle {
    id: String,
    fs_path: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct HttpRoute {
    pub id: String,
    pub http_path: String,
    pub method: String,
    pub js_entry_point: String,
    pub js_export: String,
}

async fn deploy_bundles(deploy: Deployment) -> Result<()> {
    let working_dir = Path::new(crate::DARX_SERVER_WORKING_DIR);
    let env_dir = working_dir.join(deploy.environment_id.as_str());
    let deploy_dir = env_dir.join(deploy.deploy_seq.to_string().as_str());

    for bundle in deploy.bundles.iter() {
        let bundle_fs_path = deploy_dir.join(bundle.fs_path.as_str());
        if let Some(parent) = bundle_fs_path.parent() {
            fs::create_dir_all(parent).await.with_context(|| {
                format!("Failed to create directory: {:?}", parent)
            })?;
        }

        // todo: do a checksum check and then skip the download.
        if bundle_fs_path.exists() {
            continue;
        }

        let mut file = fs::File::create(bundle_fs_path).await?;
        let bucket =
            new_bucket().with_context(|| format!("Failed to new bucket"))?;
        let s3_path = format!(
            "/{}/{}/{}",
            deploy.environment_id, deploy.deploy_seq, bundle.fs_path
        );
        bucket
            .get_object_to_writer(s3_path, &mut file)
            .await
            .with_context(|| {
                format!(
                    "Failed to get object from s3 to file: {}/{}",
                    deploy.deploy_seq, bundle.id
                )
            })?;
    }
    add_route(deploy.clone());
    println!(
        "deployed environment_id: {}, deploy_seq: {}",
        deploy.environment_id, deploy.deploy_seq
    );
    // println!("GLOBAL_ROUTER: {:?}", GLOBAL_ROUTER);
    Ok(())
}

fn new_bucket() -> Result<Bucket> {
    let bucket_name =
        env::var("S3_BUCKET").expect("S3_BUCKET should be configured");

    let region_code =
        env::var("S3_REGION").expect("S3_REGION should be configured");
    let access_key_id = env::var("S3_ACCESS_KEY_ID")
        .expect("S3_ACCESS_KEY should be configured");
    let secret_access_key = env::var("S3_SECRET_ACCESS_KEY")
        .expect("S3_SECRET_ACCESS_KEY should be configured");

    let credentials = Credentials::new(
        Some(&access_key_id),
        Some(&secret_access_key),
        None,
        None,
        None,
    )?;

    let endpoint = format!("s3.{}.amazonaws.com", region_code);
    let region = Region::Custom {
        region: region_code,
        endpoint,
    };
    let bucket = Bucket::new(bucket_name.as_str(), region, credentials)?;
    Ok(bucket)
}

static GLOBAL_ROUTER: Lazy<DashMap<String, Vec<Deployment>>> =
    Lazy::new(|| DashMap::new());

pub fn match_route(
    environment_id: &str,
    func_url: &str,
    method: &str,
) -> Option<(i64, HttpRoute)> {
    println!("match_route: func_url: {}", func_url);
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

fn add_route(mut deployment: Deployment) {
    let env_id = deployment.environment_id.clone();
    deployment
        .http_routes
        .sort_by(|a, b| a.http_path.cmp(&b.http_path));
    let mut entry = GLOBAL_ROUTER.entry(env_id).or_insert_with(|| Vec::new());
    entry.insert(0, deployment.clone());
    entry.sort_by(|a, b| b.deploy_seq.cmp(&a.deploy_seq));
}
