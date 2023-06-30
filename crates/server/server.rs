use anyhow::{anyhow, Context, Result};
use axum::extract::{Host, Path as AxumPath, State};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::{get, post};
use axum::{Json, Router};
use darx_api::ApiError;
use dotenvy::dotenv;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;

use crate::control_plane::{
    download_bundles, init_global_router, match_route, start_cmd_handler,
};
use crate::worker::{WorkerEvent, WorkerPool};
use crate::DARX_SERVER_WORKING_DIR;
use darx_api::CreatProjectRequest;
use darx_db::DBType;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::sync::oneshot;
use tokio::task::JoinSet;

pub async fn run_server(port: u16, projects_dir: &str) -> Result<()> {
    let projects_dir = fs::canonicalize(projects_dir).await?;
    let projects_dir = projects_dir.join(crate::DARX_SERVER_WORKING_DIR);
    fs::create_dir_all(projects_dir.as_path()).await?;

    #[cfg(debug_assertions)]
    dotenv().expect("failed to load .env file");

    init_global_router().await?;

    download_bundles().await?;

    let mut join_set: JoinSet<Result<()>> = JoinSet::new();

    let worker_pool = WorkerPool::new();
    let server_state = Arc::new(ServerState {
        worker_pool,
        projects_dir,
    });

    join_set.spawn(async move {
        let app = Router::new()
            .route("/", get(|| async { "darx api healthy." }))
            .route("/invoke/*func_url", post(invoke_function))
            // .route("/deploy_schema", post(deploy_schema))
            // .route("/deploy_functions", post(deploy_functions))
            .with_state(server_state);

        let socket_addr = format!("127.0.0.1:{}", port);

        println!("darx server listen on {}", socket_addr);

        axum::Server::bind(&socket_addr.parse().unwrap())
            .serve(app.into_make_service())
            .await?;
        Ok(())
    });

    join_set.spawn(start_cmd_handler());

    while let Some(result) = join_set.join_next().await {
        result?.with_context(|| "Failed to exit")?;
    }

    Ok(())
}

async fn invoke_function(
    State(server_state): State<Arc<ServerState>>,
    host: Host,
    AxumPath(func_url): AxumPath<String>,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let domain = host.0;
    let env_id = extract_env_id(domain.as_str())?;

    let (deploy_seq, route) =
        match_route(env_id.as_str(), func_url.as_str(), "POST").ok_or(
            ApiError::FunctionNotFound(format!(
                "domain: {}, env_id: {}",
                domain, env_id
            )),
        )?;

    let (resp_tx, resp_rx) = oneshot::channel();
    let raw_value = serde_json::value::RawValue::from_string(req.to_string())
        .with_context(|| {
        format!(
            "failed to serialize request body for function '{}'",
            func_url
        )
    })?;
    let working_dir = Path::new(DARX_SERVER_WORKING_DIR);
    let bundle_dir = deployment_dir(working_dir, env_id.as_str(), deploy_seq)
        .canonicalize()?;
    let event = WorkerEvent::InvokeFunction {
        env_id,
        deploy_seq,
        bundle_dir,
        js_entry_point: route.js_entry_point,
        js_export: route.js_export,
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
            func_url
        )
    })?;
    Ok(Json(result.unwrap()))
}
//
// async fn deploy_schema(
//     State(server_state): State<Arc<ServerState>>,
//     AxumPath(project_id): AxumPath<String>,
//     Json(req): Json<DeploySchemaRequest>,
// ) -> Result<Json<DeploySchemaResponse>, ApiError> {
//     let db_type = darx_db::get_db_type(project_id.as_str())?;
//     let deployment_id = match db_type {
//         DBType::MySQL => {
//             darx_db::mysql::deploy_schema(project_id.as_str(), req.migrations)
//                 .await?
//         }
//         DBType::Sqlite => {
//             darx_db::sqlite::deploy_schema(project_id.as_str(), req.migrations)
//                 .await?
//         }
//     };
//     Ok(Json(DeploySchemaResponse { deployment_id }))
// }

// async fn deploy_functions(
//     State(server_state): State<Arc<ServerState>>,
//     AxumPath(project_id): AxumPath<String>,
//     Json(req): Json<DeployFunctionsRequest>,
// ) -> Result<Json<DeployFunctionsResponse>, ApiError> {
//     let db_type = darx_db::get_db_type(project_id.as_str())?;
//     let deployment_id = match db_type {
//         DBType::MySQL => {
//             darx_db::mysql::deploy_functions(
//                 project_id.as_str(),
//                 &req.bundles,
//                 &req.bundle_meta,
//             )
//             .await?
//         }
//         DBType::Sqlite => {
//             darx_db::sqlite::deploy_functions(project_id.as_str(), &req.bundles)
//                 .await?
//         }
//     };
//     Ok(Json(DeployFunctionsResponse { deployment_id }))
// }

fn deployment_dir(
    working_dir: &Path,
    env_id: &str,
    deploy_seq: i64,
) -> PathBuf {
    working_dir
        .join(env_id)
        .join(deploy_seq.to_string().as_str())
}

// fn project_db_file(db_dir: &Path, project_id: &str) -> PathBuf {
//     db_dir.join(project_id)
// }
//
// fn project_db_conn(
//     db_dir: &Path,
//     project_id: &str,
// ) -> Result<rusqlite::Connection, ApiError> {
//     let db_file = project_db_file(db_dir, project_id);
//     rusqlite::Connection::open(db_file.as_path()).map_err(|e| {
//         ApiError::Internal(anyhow!(
//             "failed to open sqlite file: {}, error: {}",
//             db_file.to_str().unwrap(),
//             e
//         ))
//     })
// }

// async fn load_bundles_from_db(
//     project_id: &str,
//     deployment_id: &str,
// ) -> Result<(Vec<Bundle>, serde_json::Value)> {
//     let db_type = darx_db::get_db_type(project_id)?;
//     match db_type {
//         DBType::MySQL => {
//             darx_db::mysql::load_bundles_from_db(project_id, deployment_id)
//                 .await
//         }
//         DBType::Sqlite => {
//             unimplemented!()
//         }
//     }
// }

// async fn create_local_bundles(
//     projects_dir: &Path,
//     project_id: &str,
//     deployment_id: DeploymentId,
//     bundles: &Vec<Bundle>,
//     bundle_meta: &serde_json::Value,
// ) -> Result<()> {
//     let project_dir = deployment_dir(projects_dir, project_id);
//     fs::create_dir_all(project_dir.as_path())
//         .await
//         .with_context(|| {
//             format!(
//                 "failed to create directory: {}",
//                 project_dir.to_str().unwrap()
//             )
//         })?;
//
//     // setup a clean temporary path.
//
//     let tmp_dir = project_dir.join("tmp");
//     fs::create_dir_all(tmp_dir.as_path()).await?;
//     fs::remove_dir_all(tmp_dir.as_path()).await?;
//     fs::create_dir_all(tmp_dir.as_path())
//         .await
//         .with_context(|| {
//             format!(
//                 "failed to create tmp directory: {}",
//                 tmp_dir.to_str().unwrap()
//             )
//         })?;
//
//     let bundle_meta_file = tmp_dir.join("meta.json");
//     let mut f = File::create(bundle_meta_file.as_path())
//         .await
//         .with_context(|| {
//             format!(
//                 "failed to create file: {}",
//                 bundle_meta_file.to_str().unwrap()
//             )
//         })?;
//     f.write_all(bundle_meta.to_string().as_ref())
//         .await
//         .with_context(|| {
//             format!(
//                 "failed to write file: {}",
//                 bundle_meta_file.to_str().unwrap()
//             )
//         })?;
//
//     for bundle in bundles {
//         let bundle_path = tmp_dir.join(bundle.path.as_str());
//         if let Some(parent) = bundle_path.parent() {
//             fs::create_dir_all(parent).await?;
//         }
//         let mut f =
//             File::create(bundle_path.as_path()).await.with_context(|| {
//                 format!(
//                     "failed to create file: {}",
//                     bundle_path.to_str().unwrap()
//                 )
//             })?;
//
//         f.write_all(bundle.code.as_ref()).await.with_context(|| {
//             format!("failed to write file: {}", bundle_path.to_str().unwrap())
//         })?;
//     }
//
//     let meta_path = tmp_dir.join("meta.json");
//     let mut f = File::create(meta_path.as_path()).await.with_context(|| {
//         format!("failed to create file: {}", meta_path.to_str().unwrap())
//     })?;
//
//     f.write_all(bundle_meta.to_string().as_ref()).await?;
//
//     // rename the temporary directory to final directory.
//     let deploy_path = project_dir.join(deployment_id.to_string().as_str());
//     fs::rename(tmp_dir.as_path(), deploy_path.as_path()).await?;
//
//     // delete the temporary directory.
//     fs::remove_dir_all(tmp_dir.as_path()).await?;
//     Ok(())
// }

#[derive(Deserialize)]
struct ConnectDBRequest {
    project_id: String,
    db_type: DBType,
    db_url: String,
}

struct ServerState {
    worker_pool: WorkerPool,
    projects_dir: PathBuf,
}

fn extract_env_id(domain: &str) -> Result<String> {
    let mut parts = domain.split('.');
    let env_id = parts.next().ok_or_else(|| {
        ApiError::DomainNotFound(format!("invalid domain: {}", domain))
    })?;
    Ok(env_id.to_string())
}
