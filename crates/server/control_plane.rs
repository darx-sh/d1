use anyhow::{Context, Result};
use dashmap::DashMap;
use dotenv::dotenv;
use futures_util::StreamExt as _;
use once_cell::sync::Lazy;
use redis::AsyncCommands;
use s3::creds::Credentials;
use s3::{Bucket, Region};
use serde::Deserialize;
use std::env;
use std::path::Path;
use tokio::fs;
use tokio::task::JoinSet;

pub async fn start_cmd_handler() -> Result<()> {
    #[cfg(debug_assertions)]
    dotenv().expect("failed to load .env file");

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
        deploy_bundles(&deployment).await.with_context(|| {
            format!("Failed to deploy bundles: {:?}", deployment)
        })?;
    }

    panic!("no more messages");

    Ok(())
}

#[derive(Clone, Debug, Deserialize)]
struct Deployment {
    project_id: String,
    environment_id: String,
    deployment_id: String,
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
    pub http_path: String,
    pub method: String,
    pub js_entry_point: String,
    pub js_export: String,
}

async fn deploy_bundles(deploy: &Deployment) -> Result<()> {
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
                    deploy.deployment_id, bundle.id
                )
            })?;
    }
    add_route(&deploy);
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
    if let Some(entry) = GLOBAL_ROUTER.get(environment_id) {
        let mut cur_deploy = entry[0].clone();
        // sort a deployment's route based on url
        cur_deploy
            .http_routes
            .sort_by(|a, b| a.http_path.cmp(&b.http_path));
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

fn add_route(deployment: &Deployment) {
    let env_id = deployment.environment_id.clone();
    let mut routes = GLOBAL_ROUTER.entry(env_id).or_insert_with(|| Vec::new());
    routes.insert(0, deployment.clone());
    routes.sort_by(|a, b| a.deploy_seq.cmp(&b.deploy_seq));
}
