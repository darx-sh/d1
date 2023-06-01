use crate::ApiError;
use anyhow::{anyhow, Context, Result};
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;

use crate::worker::{WorkerEvent, WorkerPool};
use darx_utils::create_db_pool;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::sync::oneshot;

pub async fn run_server(
    port: u16,
    projects_dir: &str,
    sqlite_dir: &str,
) -> Result<()> {
    let projects_dir = fs::canonicalize(projects_dir).await?;
    let sqlite_dir = fs::canonicalize(sqlite_dir).await?;

    fs::create_dir_all(projects_dir.as_path()).await?;

    // sqlite dir needs to exist before we run the server.
    // This is because we need a mounted litefs directory.
    if !sqlite_dir.exists() {
        return Err(anyhow!(
            "sqlite directory does not exist: {}",
            sqlite_dir.display()
        ));
    }

    let db_pool = create_db_pool();
    let worker_pool = WorkerPool::new();
    let server_state = Arc::new(ServerState {
        mysql_pool: db_pool,
        worker_pool,
        projects_dir,
        sqlite_dir,
    });

    let app = Router::new()
        .route("/", get(|| async { "darx api healthy." }))
        .route("/create_project", post(create_project))
        .route("/invoke/:function_name", post(invoke_function))
        .route("/deploy_schema", post(deploy_schema))
        .route("/deploy_functions", post(deploy_functions))
        .route("/rollback_functions", post(rollback_functions))
        .route("/deployments/:deployment_id", get(get_deployment))
        .route("/deployments", get(list_deployments))
        .with_state(server_state);

    let socket_addr = format!("127.0.0.1:{}", port);
    axum::Server::bind(&socket_addr.parse().unwrap())
        .serve(app.into_make_service())
        .await?;
    Ok(())
}

async fn create_project(
    State(server_state): State<Arc<ServerState>>,
    Json(req): Json<CreatProjectRequest>,
) -> Result<StatusCode, ApiError> {
    let project_dir = server_state.projects_dir.join(&req.project_id);
    fs::create_dir(&project_dir).await?;

    let sqlite_file = server_state
        .sqlite_dir
        .join(format!("{}.db", &req.project_id));

    let conn = rusqlite::Connection::open(sqlite_file.as_path())?;
    conn.pragma_update(None, "journal_mode", &"WAL")?;
    conn.pragma_update(None, "synchronous", &"NORMAL")?;

    let schema_sql = include_str!("schema.sql");
    conn.execute_batch(schema_sql)?;
    Ok(StatusCode::CREATED)
}

async fn invoke_function(
    State(server_state): State<Arc<ServerState>>,
    Path(func_name): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let db_pool = (&server_state).mysql_pool.clone();
    let (resp_tx, resp_rx) = oneshot::channel();
    let event = WorkerEvent::InvokeFunction {
        db_pool,
        project_dir: project_dir().to_string(),
        func_name: func_name.clone(),
        params: Default::default(),
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

async fn deploy_schema(
    Json(req): Json<DeploySchemaRequest>,
) -> Result<Json<DeploySchemaResponse>, ApiError> {
    todo!()
}

async fn get_deployment(
    Path(deployment_id): Path<i64>,
) -> Result<Json<GetDeploymentResponse>, ApiError> {
    todo!()
}

async fn deploy_functions(
    Json(body): Json<DeployFunctionsRequest>,
) -> Result<Json<DeployFunctionsResponse>, ApiError> {
    todo!()
}

async fn rollback_functions(
    Json(body): Json<RollbackFunctionsRequest>,
) -> Result<Json<RollbackFunctionsResponse>, ApiError> {
    todo!()
}

async fn create_draft_module(
    Json(req): Json<CreateModuleRequest>,
) -> Result<StatusCode, ApiError> {
    // save file to tenant's directory
    let tenant_dir_path = PathBuf::from(project_dir());
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
        format!("{}/{}/{}", project_dir(), dir, req.file_name)
    } else {
        format!("{}/{}", project_dir(), req.file_name)
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

async fn list_deployments() -> Result<Json<Vec<GetDeploymentResponse>>, ApiError>
{
    todo!()
}

// fn project_dir() -> String {
//     format!(
//         "{}/{}",
//         env!("CARGO_MANIFEST_DIR"),
//         "examples/projects/1234567"
//     )
// }

fn projects_dir() -> String {
    format!("{}/{}", env!("CARGO_MANIFEST_DIR"), "examples/projects")
}

fn project_dir(project_id: &str) -> String {
    format!("{}/{}", projects_dir(), project_id)
}

#[derive(Deserialize)]
struct CreateModuleRequest {
    dir: Option<String>,
    file_name: String,
    code: String,
}

#[derive(Deserialize)]
struct CreatProjectRequest {
    // project_id should unique in the system.
    project_id: String,
}

#[derive(Deserialize)]
struct DeploySchemaRequest {
    sql: String,
}

#[derive(Serialize)]
struct DeploySchemaResponse {
    deployment_id: i64,
}

#[derive(Deserialize)]
struct DeployFunctionsRequest {
    project_id: String,
    modules: Vec<Bundle>,
    description: Option<String>,
}

#[derive(Serialize)]
struct DeployFunctionsResponse {
    deployment_id: i64,
}

#[derive(Serialize)]
struct GetDeploymentResponse {
    deploy_type: DeploymentType,
    status: DeploymentStatus,
}

#[derive(Deserialize)]
struct RollbackFunctionsRequest {
    target_deployment_id: i64,
}

/// Rollback will create another deployment [`new_deployment_id`].
#[derive(Serialize)]
struct RollbackFunctionsResponse {
    new_deployment_id: i64,
}

#[derive(Deserialize)]
struct Bundle {
    path: String,
    code: String,
}

#[derive(Serialize)]
enum DeploymentType {
    Schema,
    Functions,
}

#[derive(Serialize)]
enum DeploymentStatus {
    OnGoing,
    Done,
    Failed,
}

struct ServerState {
    // will use sqlite by default.
    mysql_pool: mysql_async::Pool,
    worker_pool: WorkerPool,
    projects_dir: PathBuf,
    sqlite_dir: PathBuf,
}
