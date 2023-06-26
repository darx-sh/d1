use anyhow::{Context, Result};
use dashmap::DashMap;
use dotenv::dotenv;
use futures_util::StreamExt as _;
use redis::AsyncCommands;
use s3::creds::Credentials;
use s3::Bucket;
use serde::Deserialize;
use std::env;
use std::path::Path;
use tokio::fs;
use tokio::task::JoinSet;

pub async fn start_control_plane_handler() -> Result<()> {
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
        println!("Got message: {:?}", deployment);
        deploy_bundles(&deployment).await.with_context(|| {
            format!("Failed to deploy bundles: {:?}", deployment)
        })?;
    }

    panic!("no more messages");

    Ok(())
}

#[derive(Deserialize, Debug)]
struct Deployment {
    project_id: String,
    environment_id: String,
    deployment_id: String,
    bundles: Vec<Bundle>,
    http_routes: Vec<HttpRoute>,
}

#[derive(Deserialize, Debug)]
struct Bundle {
    id: String,
    fs_path: String,
}

#[derive(Deserialize, Debug)]
struct HttpRoute {
    http_path: String,
    method: String,
    js_entry_point: String,
    js_export: String,
}

type GlobalRouter = DashMap<String, Deployment>;

async fn deploy_bundles(deploy: &Deployment) -> Result<()> {
    let working_dir = Path::new(crate::DARX_SERVER_WORKING_DIR);
    let env_dir = working_dir.join(deploy.environment_id.as_str());

    for bundle in deploy.bundles.iter() {
        let bundle_fs_path = env_dir.join(bundle.fs_path.as_str());
        if let Some(parent) = bundle_fs_path.parent() {
            fs::create_dir_all(parent).await.with_context(|| {
                format!("Failed to create directory: {:?}", parent)
            })?;
        }

        let mut file = fs::File::create(bundle_fs_path).await?;
        let bucket =
            new_bucket().with_context(|| format!("Failed to new bucket"))?;
        let s3_path = format!("/{}/{}", deploy.deployment_id, bundle.id);
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
    Ok(())
}

fn new_bucket() -> Result<Bucket> {
    let bucket_name =
        env::var("S3_BUCKET").expect("S3_BUCKET should be configured");

    let region = env::var("S3_REGION").expect("S3_REGION should be configured");
    let access_key_id = env::var("S3_ACCESS_KEY_ID")
        .expect("S3_ACCESS_KEY should be configured");
    let secret_access_key = env::var("S3_SECRET_ACCESS_KEY")
        .expect("S3_SECRET_ACCESS_KEY should be configured");
    println!(
        "access_key_id: {}, secret_access_key: {}",
        access_key_id, secret_access_key
    );

    let credentials = Credentials::new(
        Some(&access_key_id),
        Some(&secret_access_key),
        None,
        None,
        None,
    )?;
    let bucket =
        Bucket::new(bucket_name.as_str(), region.parse()?, credentials)?;
    Ok(bucket)
}
