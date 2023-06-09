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
use darx_db::{
    Bundle, DBType, DeploymentId, DeploymentStatus, DeploymentType, Migration,
};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::sync::oneshot;

pub async fn run_server(port: u16, projects_dir: &str) -> Result<()> {
    let projects_dir = fs::canonicalize(projects_dir).await?;
    // let sqlite_dir = fs::canonicalize(sqlite_dir).await?;
    let projects_dir = projects_dir.join("darx_projects");
    fs::create_dir_all(projects_dir.as_path()).await?;

    // sqlite dir needs to exist before we run the server.
    // This is because we need a mounted litefs directory.
    // if !sqlite_dir.exists() {
    //     return Err(anyhow!(
    //         "sqlite directory does not exist: {}",
    //         sqlite_dir.display()
    //     ));
    // }

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
    let (resp_tx, resp_rx) = oneshot::channel();
    let raw_value = serde_json::value::RawValue::from_string(req.to_string())
        .with_context(|| {
        format!(
            "failed to serialize request body for function '{}'",
            func_name
        )
    })?;
    let event = WorkerEvent::InvokeFunction {
        project_id: project_id.clone(),
        bundle_dir: project_bundle_dir(
            server_state.projects_dir.as_path(),
            project_id.as_str(),
        ),
        func_name: func_name.clone(),
        params: raw_value,
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
    let db_type = darx_db::get_db_type(project_id.as_str())?;
    let deployment_id = match db_type {
        DBType::MySQL => {
            darx_db::mysql::deploy_schema(project_id.as_str(), req.migrations)
                .await?
        }
        DBType::Sqlite => {
            darx_db::sqlite::deploy_schema(project_id.as_str(), req.migrations)
                .await?
        }
    };
    Ok(Json(DeploySchemaResponse { deployment_id }))
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
    let db_type = darx_db::get_db_type(project_id.as_str())?;
    let deployment_id = match db_type {
        DBType::MySQL => {
            darx_db::mysql::deploy_functions(
                project_id.as_str(),
                &req.bundles,
                &req.bundle_meta,
            )
            .await?
        }
        DBType::Sqlite => {
            darx_db::sqlite::deploy_functions(project_id.as_str(), &req.bundles)
                .await?
        }
    };
    Ok(Json(DeployFunctionsResponse { deployment_id }))
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

async fn load_bundles_from_db(
    project_id: &str,
    deployment_id: &str,
) -> Result<(Vec<Bundle>, serde_json::Value)> {
    let db_type = darx_db::get_db_type(project_id)?;
    match db_type {
        DBType::MySQL => {
            darx_db::mysql::load_bundles_from_db(project_id, deployment_id)
                .await
        }
        DBType::Sqlite => {
            unimplemented!()
        }
    }
}

async fn create_local_bundles(
    projects_dir: &Path,
    project_id: &str,
    deployment_id: DeploymentId,
    bundles: &Vec<Bundle>,
    bundle_meta: &serde_json::Value,
) -> Result<()> {
    let project_dir = project_bundle_dir(projects_dir, project_id);
    fs::create_dir_all(project_dir.as_path())
        .await
        .with_context(|| {
            format!(
                "failed to create directory: {}",
                project_dir.to_str().unwrap()
            )
        })?;

    // setup a clean temporary path.

    let tmp_dir = project_dir.join("tmp");
    fs::create_dir_all(tmp_dir.as_path()).await?;
    fs::remove_dir_all(tmp_dir.as_path()).await?;
    fs::create_dir_all(tmp_dir.as_path())
        .await
        .with_context(|| {
            format!(
                "failed to create tmp directory: {}",
                tmp_dir.to_str().unwrap()
            )
        })?;

    let bundle_meta_file = tmp_dir.join("meta.json");
    let mut f = File::create(bundle_meta_file.as_path())
        .await
        .with_context(|| {
            format!(
                "failed to create file: {}",
                bundle_meta_file.to_str().unwrap()
            )
        })?;
    f.write_all(bundle_meta.to_string().as_ref())
        .await
        .with_context(|| {
            format!(
                "failed to write file: {}",
                bundle_meta_file.to_str().unwrap()
            )
        })?;

    for bundle in bundles {
        let bundle_path = tmp_dir.join(bundle.path.as_str());
        if let Some(parent) = bundle_path.parent() {
            fs::create_dir_all(parent).await?;
        }
        let mut f =
            File::create(bundle_path.as_path()).await.with_context(|| {
                format!(
                    "failed to create file: {}",
                    bundle_path.to_str().unwrap()
                )
            })?;

        f.write_all(bundle.code.as_ref()).await.with_context(|| {
            format!("failed to write file: {}", bundle_path.to_str().unwrap())
        })?;
    }

    let meta_path = tmp_dir.join("meta.json");
    let mut f = File::create(meta_path.as_path()).await.with_context(|| {
        format!("failed to create file: {}", meta_path.to_str().unwrap())
    })?;

    f.write_all(bundle_meta.to_string().as_ref()).await?;

    // rename the temporary directory to final directory.
    let deploy_path = project_dir.join(deployment_id.to_string().as_str());
    fs::rename(tmp_dir.as_path(), deploy_path.as_path()).await?;

    // delete the temporary directory.
    fs::remove_dir_all(tmp_dir.as_path()).await?;
    Ok(())
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
    db_type: DBType,
    db_url: String,
}

#[derive(Deserialize)]
struct ConnectDBRequest {
    project_id: String,
    db_type: DBType,
    db_url: String,
}

#[derive(Deserialize)]
pub struct DeploySchemaRequest {
    migrations: Vec<Migration>,
}

#[derive(Serialize)]
struct DeploySchemaResponse {
    deployment_id: DeploymentId,
}

#[derive(Deserialize)]
struct DeployFunctionsRequest {
    bundles: Vec<Bundle>,
    bundle_meta: serde_json::Value,
    description: Option<String>,
}

#[derive(Serialize)]
struct DeployFunctionsResponse {
    deployment_id: DeploymentId,
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

struct ServerState {
    worker_pool: WorkerPool,
    projects_dir: PathBuf,
}
