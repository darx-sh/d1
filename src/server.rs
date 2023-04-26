use crate::api_mgr::ApiError;
use anyhow::{Context, Result};
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;

use crate::utils::create_db_pool;
use crate::worker::{WorkerEvent, WorkerPool};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::sync::oneshot;

pub async fn run_server() -> Result<()> {
    let db_pool = create_db_pool();
    let worker_pool = WorkerPool::new();
    let server_state = Arc::new(ServerState {
        db_pool,
        worker_pool,
    });

    let app = Router::new()
        .route(
            "/d/f/:function_name",
            get(invoke_func_get).post(invoke_func_post),
        )
        .route(
            "/c/draft/modules",
            get(list_draft_modules).post(create_draft_module),
        )
        .route(
            "/c/preview/f/:function_name",
            get(invoke_func_preview_get).post(invoke_func_preview_post),
        )
        .route("/c/deploy", get(list_deployments).post(create_deployment))
        .with_state(server_state);

    axum::Server::bind(&"127.0.0.1:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await?;
    Ok(())
}

async fn invoke_func_get(
    State(server_state): State<Arc<ServerState>>,
    Query(params): Query<HashMap<String, String>>,
    Path(func_name): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    Ok(Json(json!("not implemented yet")))
}

async fn invoke_func_post(
    State(server_state): State<Arc<ServerState>>,
    Path(func_name): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, ApiError> {
    Ok(Json(json!("not implemented yet")))
}

async fn invoke_func_preview_get(
    State(server_state): State<Arc<ServerState>>,
    Query(params): Query<HashMap<String, String>>,
    Path(func_name): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let db_pool = (&server_state).db_pool.clone();
    let (resp_tx, resp_rx) = oneshot::channel();
    let event = WorkerEvent::InvokeFunction {
        db_pool,
        tenant_dir: tenant_dir().to_string(),
        func_name: func_name.clone(),
        params,
        resp: resp_tx,
    };
    server_state
        .worker_pool
        .send(event)
        .with_context(|| format!("failed to send event to worker pool"))?;

    let result = resp_rx.await.with_context(|| {
        format!(
            "failed to receive response from worker pool for function '{}'",
            func_name
        )
    })?;
    Ok(Json(result.unwrap()))
}

async fn invoke_func_preview_post(
    State(server_state): State<Arc<ServerState>>,
    Path(func_name): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, ApiError> {
    todo!()
}

async fn list_draft_modules() -> Result<Json<serde_json::Value>, ApiError> {
    todo!()
}

async fn create_draft_module(
    Json(req): Json<CreateModuleRequest>,
) -> Result<StatusCode, ApiError> {
    // save file to tenant's directory
    let tenant_dir_path = PathBuf::from(tenant_dir());
    fs::create_dir_all(tenant_dir_path.as_path())
        .await
        .with_context(|| {
            format!(
                "failed to create directory: {}",
                tenant_dir_path.to_str().unwrap()
            )
        })?;

    let file_full_path = if let Some(dir) = req.dir {
        let mut file_dir_path = tenant_dir_path.clone();
        file_dir_path.push(&dir);
        fs::create_dir_all(file_dir_path.as_path())
            .await
            .with_context(|| {
                format!(
                    "failed to create directory: {}",
                    file_dir_path.to_str().unwrap()
                )
            })?;
        format!("{}/{}/{}", tenant_dir(), dir, req.file_name)
    } else {
        format!("{}/{}", tenant_dir(), req.file_name)
    };

    let mut f = File::create(PathBuf::from(file_full_path.as_str()))
        .await
        .with_context(|| {
            format!("failed to create module file: {}", file_full_path.as_str())
        })?;

    f.write_all(req.code.as_ref()).await.with_context(|| {
        format!("failed to write module file: {}", file_full_path.as_str())
    })?;

    Ok(StatusCode::CREATED)
}

async fn list_deployments() -> Result<StatusCode, ApiError> {
    todo!()
}

async fn create_deployment() -> Result<StatusCode, ApiError> {
    todo!()
}

fn tenant_dir() -> String {
    format!(
        "{}/{}",
        env!("CARGO_MANIFEST_DIR"),
        "examples/tenants/1234567"
    )
}

#[derive(Deserialize)]
struct CreateModuleRequest {
    dir: Option<String>,
    file_name: String,
    code: String,
}

struct ServerState {
    db_pool: mysql_async::Pool,
    worker_pool: WorkerPool,
}
