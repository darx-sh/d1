mod route_builder;

use crate::route_builder::build_route;
use anyhow::{Context, Result};
use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};
use darx_api::{ApiError, BundleRsp, PrepareDeployReq, PrepareDeployRsp};
use dotenvy::dotenv;
use nanoid::nanoid;
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;

pub async fn run_server(socket_addr: SocketAddr) -> Result<()> {
    #[cfg(debug_assertions)]
    dotenv().expect("Failed to load .env file");

    let db_pool = sqlx::MySqlPool::connect(
        env::var("DATABASE_URL")
            .expect("DATABASE_URL should be configured")
            .as_str(),
    )
    .await
    .context("Failed to connect database")?;
    let server_state = Arc::new(ServerState { db_pool });

    let app = Router::new()
        .route("/", get(|| async { "control plane healthy." }))
        .route("/prepare_deploy", post(prepare_deploy))
        .route("/deploy_bundle_status", post(update_bundle_status))
        .with_state(server_state);

    tracing::info!("listen on {}", socket_addr);
    axum::Server::bind(&socket_addr)
        .serve(app.into_make_service())
        .await
        .context("Failed to start control plane server")?;
    Ok(())
}

async fn prepare_deploy(
    State(server_state): State<Arc<ServerState>>,
    Json(req): Json<PrepareDeployReq>,
) -> Result<Json<PrepareDeployRsp>, ApiError> {
    let db_pool = &server_state.db_pool;
    let mut routes = vec![];
    for meta in req.metas.iter() {
        for js_export in meta.exports.iter() {
            let route =
                build_route(meta.entry_point.as_str(), js_export.as_str())?;
            routes.push(route);
        }
    }
    let tx = db_pool
        .begin()
        .await
        .context("Failed to start database transaction")?;

    let env = sqlx::query!(
        "SELECT next_deploy_seq FROM envs WHERE id = ? FOR UPDATE",
        req.env_id
    )
    .fetch_optional(db_pool)
    .await
    .context("Failed to find env")?
    .ok_or(ApiError::EnvNotFound(req.env_id.clone()))?;

    sqlx::query!(
        "UPDATE envs SET next_deploy_seq = next_deploy_seq + 1 WHERE id = ?",
        req.env_id
    )
    .execute(db_pool)
    .await
    .context("Failed to update envs table")?;

    let deploy_seq = env.next_deploy_seq + 1;

    // create new deploy
    // todo: retry to avoid id collision
    let deploy_id = new_nano_id();
    sqlx::query!(
        "INSERT INTO deploys (id, updated_at, tag, description, env_id, deploy_seq, bundle_cnt) VALUES (?, CURRENT_TIMESTAMP(3), ?, ?, ?, ?, ?)",
        deploy_id,
        req.tag,
        req.description,
        req.env_id,
        deploy_seq,
        req.bundles.len() as i64,
    )
    .execute(db_pool)
    .await
    .context("Failed to insert into deploys table")?;

    // create new bundle
    let mut bundle_ids = vec![];
    for bundle in req.bundles.iter() {
        let bundle_id = new_nano_id();
        sqlx::query!(
            "INSERT INTO bundles (id, updated_at, bytes, deploy_id, fs_path) VALUES (?, CURRENT_TIMESTAMP(3), ?, ?, ?)",
            bundle_id,
            bundle.bytes,
            deploy_id,
            bundle.fs_path,
        )
        .execute(db_pool)
        .await
        .context("Failed to insert into bundles table")?;
        bundle_ids.push(bundle_id);
    }

    // create new route
    for route in routes.iter() {
        let route_id = new_nano_id();
        sqlx::query!(
            "INSERT INTO http_routes (id, updated_at, method, js_entry_point, js_export, deploy_id, http_path) VALUES (?, CURRENT_TIMESTAMP(3), ?, ?, ?, ?, ?)",
            route_id,
            "POST",
            route.js_entry_point,
            route.js_export,
            deploy_id,
            route.http_path
        ).execute(db_pool).await.context("Failed to insert into http_routes table")?;
    }
    tx.commit()
        .await
        .context("Failed to commit database transaction")?;
    // return data plan url to client
    let mut bundles = vec![];
    for (bundle, id) in req.bundles.iter().zip(bundle_ids.iter()) {
        let url = format!(
            "{}/deploy_bundle/{}/{}",
            env::var("DATA_PLANE_URL")
                .expect("DATA_PLANE_URL should be configured"),
            req.env_id,
            deploy_seq,
        );
        bundles.push(BundleRsp {
            id: id.to_string(),
            fs_path: bundle.fs_path.clone(),
            upload_url: url,
            upload_method: "POST".to_string(),
        });
    }
    Ok(Json(PrepareDeployRsp { deploy_id, bundles }))
}

async fn update_bundle_status() {
    todo!()
}

struct ServerState {
    db_pool: sqlx::MySqlPool,
}

fn new_nano_id() -> String {
    let alphabet = "0123456789abcdefghijklmnopqrstuvwxyz";
    let chars = alphabet.chars().collect::<Vec<_>>();
    nanoid!(12, &chars)
}
