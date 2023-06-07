use crate::ApiError;
use anyhow::{anyhow, Context, Result};
use axum::extract::{Path as AxumPath, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use rusqlite::types::{
    FromSql, FromSqlError, FromSqlResult, ToSqlOutput, Value, ValueRef,
};
use rusqlite::{params, ToSql};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;

use crate::worker::{WorkerEvent, WorkerPool};
use darx_db::mysql::MySqlPool;
// use darx_db::Database;
use darx_utils::test_mysql_db_pool;
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
    let projects_dir = projects_dir.join("darx_projects");
    fs::create_dir_all(projects_dir.as_path()).await?;

    // sqlite dir needs to exist before we run the server.
    // This is because we need a mounted litefs directory.
    if !sqlite_dir.exists() {
        return Err(anyhow!(
            "sqlite directory does not exist: {}",
            sqlite_dir.display()
        ));
    }

    let db_pool = test_mysql_db_pool();

    let worker_pool = WorkerPool::new();
    let server_state = Arc::new(ServerState {
        worker_pool,
        projects_dir,
    });

    let app = Router::new()
        .route("/", get(|| async { "darx api healthy." }))
        .route("/create_project", post(create_project))
        .route(
            "/app/:project_id/invoke/:function_name",
            post(invoke_function),
        )
        .route("/app/:project_id/deploy_schema", post(deploy_schema))
        .route("/app/:project_id/deploy_functions", post(deploy_functions))
        .route(
            "/app/:project_id/rollback_functions",
            post(rollback_functions),
        )
        .route(
            "/app/:project_id/deployments/:deployment_id",
            get(get_deployment),
        )
        .route("/app/:project_id/deployments", get(list_deployments))
        .with_state(server_state);

    let socket_addr = format!("127.0.0.1:{}", port);

    println!("darx server listen on {}", socket_addr);

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
    //
    // let sqlite_file =
    //     project_db_file(server_state.sqlite_dir.as_path(), &req.project_id);
    //
    // let conn = rusqlite::Connection::open(sqlite_file.as_path())?;
    // conn.pragma_update(None, "journal_mode", &"WAL")?;
    // conn.pragma_update(None, "synchronous", &"NORMAL")?;
    //
    // let schema_sql = include_str!("schema.sql");
    // conn.execute_batch(schema_sql)?;
    Ok(StatusCode::CREATED)
}

async fn invoke_function(
    State(server_state): State<Arc<ServerState>>,
    AxumPath(project_id): AxumPath<String>,
    AxumPath(func_name): AxumPath<String>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let conn_pool = MySqlPool::new("mysql://root:12345678@localhost:3306/test");
    let (resp_tx, resp_rx) = oneshot::channel();
    let event = WorkerEvent::InvokeFunction {
        project_id: project_id.clone(),
        bundle_dir: project_bundle_dir(
            server_state.projects_dir.as_path(),
            project_id.as_str(),
        ),
        func_name: func_name.clone(),
        params: Default::default(),
        resp: resp_tx,
    };
    server_state.worker_pool.send(event).map_err(|e| {
        ApiError::Internal(anyhow!(
            "failed to send request to worker pool: {}",
            e.to_string()
        ))
    })?;

    let result = resp_rx.await.with_context(|| {
        format!(
            "failed to receive response from worker pool for function '{}'",
            func_name
        )
    })?;
    Ok(Json(result.unwrap()))
}

async fn deploy_schema(
    State(server_state): State<Arc<ServerState>>,
    AxumPath(project_id): AxumPath<String>,
    Json(req): Json<DeploySchemaRequest>,
) -> Result<Json<DeploySchemaResponse>, ApiError> {
    todo!()
    // let sqlite_file =
    //     project_db_file(server_state.sqlite_dir.as_path(), project_id.as_str());
    // let conn = rusqlite::Connection::open(sqlite_file.as_path())?;
    // conn.execute(
    //     "INSERT INTO deployments (type, status) VALUES (?, ?)",
    //     params![DeploymentType::Schema, DeploymentStatus::Doing],
    // )?;
    // let deployment_id = conn.last_insert_rowid();
    // for m in req.migrations.iter() {
    //     conn.execute(
    //         "INSERT INTO db_migrations (file_name, sql, applied, deployment_id) VALUES (?, ?, ?, ?)",
    //         params![&m.file_name, &m.sql, &0, &deployment_id],
    //     )?;
    // }
    //
    // for m in req.migrations.iter() {
    //     conn.execute_batch(m.sql.as_str())?;
    //     conn.execute(
    //         "UPDATE db_migrations SET applied = 1 WHERE file_name = ?",
    //         params![m.file_name],
    //     )?;
    // }
    // conn.execute(
    //     "UPDATE deployments SET status = ? WHERE id = ?",
    //     params![DeploymentStatus::Done, deployment_id],
    // )?;
    // Ok(Json(DeploySchemaResponse { deployment_id }))
}

async fn get_deployment(
    State(server_state): State<Arc<ServerState>>,
    AxumPath(project_id): AxumPath<String>,
    AxumPath(deployment_id): AxumPath<i64>,
) -> Result<Json<GetDeploymentResponse>, ApiError> {
    todo!()
    // let conn = project_db_conn(
    //     server_state.sqlite_dir.as_path(),
    //     project_id.as_str(),
    // )?;
    // let mut stmt =
    //     conn.prepare("SELECT type, status FROM deployments WHERE id = ?")?;
    // let mut rows = stmt.query(params![deployment_id])?;
    // let row = rows.next()?.unwrap();
    // let rsp = GetDeploymentResponse {
    //     deploy_type: row.get(0)?,
    //     status: row.get(1)?,
    // };
    // Ok(Json(rsp))
}

async fn deploy_functions(
    State(server_state): State<Arc<ServerState>>,
    AxumPath(project_id): AxumPath<String>,
    Json(req): Json<DeployFunctionsRequest>,
) -> Result<Json<DeployFunctionsResponse>, ApiError> {
    todo!()
    // let conn = project_db_conn(
    //     server_state.sqlite_dir.as_path(),
    //     project_id.as_str(),
    // )?;
    // conn.execute(
    //     "INSERT INTO deployments (type, status) VALUES (?, ?)",
    //     &["functions", "start"],
    // )?;
    // let deployment_id = conn.last_insert_rowid();
    // for bundle in req.modules.iter() {
    //     conn.execute("INSERT INTO js_bundles (path, code, deployment_id) VALUES (?, ?, ?)", params![bundle.path, bundle.code, deployment_id])?;
    // }
    // conn.execute(
    //     "UPDATE deployments SET status = ? WHERE id = ?",
    //     params!["success", deployment_id],
    // )?;
    //
    // // store bundle in project's directory
    // let bundle_dir = project_bundle_dir(
    //     server_state.projects_dir.as_path(),
    //     project_id.as_str(),
    // );
    //
    // for bundle in req.modules.iter() {
    //     let bundle_file_path = PathBuf::from(bundle.path.as_str());
    //     if let Some(parent) = bundle_file_path.parent() {
    //         let mut parent_dir = bundle_dir.join(parent);
    //         std::fs::create_dir_all(parent_dir.as_path())?;
    //     }
    //     let file_path = bundle_dir.join(bundle.path.as_str());
    //     let mut file = File::create(file_path.as_path()).await?;
    //     file.write_all(bundle.code.as_bytes()).await?;
    // }
    //
    // Ok(Json(DeployFunctionsResponse { deployment_id }))
}

async fn rollback_functions(
    AxumPath(project_id): AxumPath<String>,
    Json(body): Json<RollbackFunctionsRequest>,
) -> Result<Json<RollbackFunctionsResponse>, ApiError> {
    todo!()
}

async fn create_draft_module(
    Json(req): Json<CreateModuleRequest>,
) -> Result<StatusCode, ApiError> {
    todo!()
    // save file to tenant's directory
    // let tenant_dir_path = PathBuf::from(project_dir());
    // fs::create_dir_all(tenant_dir_path.as_path())
    //     .await
    //     .with_context(|| {
    //         format!(
    //             "failed to create directory: {}",
    //             tenant_dir_path.to_str().unwrap()
    //         )
    //     })?;
    //
    // let file_full_path = if let Some(dir) = req.dir {
    //     let mut file_dir_path = tenant_dir_path.clone();
    //     file_dir_path.push(&dir);
    //     fs::create_dir_all(file_dir_path.as_path())
    //         .await
    //         .with_context(|| {
    //             format!(
    //                 "failed to create directory: {}",
    //                 file_dir_path.to_str().unwrap()
    //             )
    //         })?;
    //     format!("{}/{}/{}", project_dir(), dir, req.file_name)
    // } else {
    //     format!("{}/{}", project_dir(), req.file_name)
    // };
    //
    // let mut f = File::create(PathBuf::from(file_full_path.as_str()))
    //     .await
    //     .with_context(|| {
    //         format!("failed to create module file: {}", file_full_path.as_str())
    //     })?;
    //
    // f.write_all(req.code.as_ref()).await.with_context(|| {
    //     format!("failed to write module file: {}", file_full_path.as_str())
    // })?;
    //
    // Ok(StatusCode::CREATED)
}

async fn list_deployments() -> Result<Json<Vec<GetDeploymentResponse>>, ApiError>
{
    todo!()
}

fn project_bundle_dir(projects_dir: &Path, project_id: &str) -> PathBuf {
    projects_dir.join(project_id)
}

fn project_db_file(db_dir: &Path, project_id: &str) -> PathBuf {
    db_dir.join(project_id)
}

fn project_db_conn(
    db_dir: &Path,
    project_id: &str,
) -> Result<rusqlite::Connection, ApiError> {
    let db_file = project_db_file(db_dir, project_id);
    rusqlite::Connection::open(db_file.as_path()).map_err(|e| {
        ApiError::Internal(anyhow!(
            "failed to open sqlite file: {}, error: {}",
            db_file.to_str().unwrap(),
            e
        ))
    })
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
    migrations: Vec<Migration>,
}

#[derive(Serialize)]
struct DeploySchemaResponse {
    deployment_id: i64,
}

#[derive(Deserialize)]
struct DeployFunctionsRequest {
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

#[derive(Deserialize)]
struct Migration {
    file_name: String,
    sql: String,
}

#[derive(Serialize)]
enum DeploymentType {
    Schema,
    Functions,
}

impl FromSql for DeploymentType {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        match value.as_i64()? {
            0 => Ok(DeploymentType::Schema),
            1 => Ok(DeploymentType::Functions),
            _ => Err(FromSqlError::InvalidType),
        }
    }
}

impl ToSql for DeploymentType {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        match self {
            DeploymentType::Schema => Ok(ToSqlOutput::Owned(Value::Integer(0))),
            DeploymentType::Functions => {
                Ok(ToSqlOutput::Owned(Value::Integer(1)))
            }
        }
    }
}

#[derive(Serialize)]
enum DeploymentStatus {
    Doing,
    Done,
    Failed,
}

impl FromSql for DeploymentStatus {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        match value.as_i64()? {
            0 => Ok(DeploymentStatus::Doing),
            1 => Ok(DeploymentStatus::Done),
            2 => Ok(DeploymentStatus::Failed),
            _ => Err(FromSqlError::InvalidType),
        }
    }
}

impl ToSql for DeploymentStatus {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        match self {
            DeploymentStatus::Doing => {
                Ok(ToSqlOutput::Owned(Value::Integer(0)))
            }
            DeploymentStatus::Done => Ok(ToSqlOutput::Owned(Value::Integer(1))),
            DeploymentStatus::Failed => {
                Ok(ToSqlOutput::Owned(Value::Integer(2)))
            }
        }
    }
}

struct ServerState {
    // will use sqlite by default.
    worker_pool: WorkerPool,
    projects_dir: PathBuf,
    // sqlite_dir: PathBuf,
}
