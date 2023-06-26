use anyhow::{Context, Result};
use dotenv::dotenv;
use futures_util::StreamExt as _;
use redis::AsyncCommands;
use serde::Deserialize;
use std::env;

pub async fn start_control_plane_handler() -> Result<()> {
    #[cfg(debug_assertions)]
    dotenv().expect("failed to load .env file");

    let redis_client = redis::Client::open(
        env::var("REDIS_URL").expect("REDIS_URL should be configured"),
    )?;
    let mut pubsub = redis_client
        .get_async_connection()
        .await
        .with_context(|| format!("Failed to connect to redis"))?
        .into_pubsub();
    pubsub.subscribe("deploy").await?;
    let mut pubsub_stream = pubsub.on_message();
    while let Some(msg) = pubsub_stream.next().await {
        let payload: String = msg.get_payload()?;
        let payload = serde_json::from_str::<Deployment>(&payload).unwrap();
        println!("Got message: {:?}", payload);
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
