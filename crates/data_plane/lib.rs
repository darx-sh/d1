mod deployment;
mod worker;

use anyhow::{anyhow, Context, Result};
use axum::extract::{Host, Path as AxumPath, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use darx_api::{AddDeploymentReq, ApiError};
use dotenvy::dotenv;

use std::env;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;

use crate::worker::{WorkerEvent, WorkerPool};

use crate::deployment::{
    add_bundle_files, add_route, find_bundle_dir, init_deployments,
    match_route, DeploymentRoute,
};

use tokio::sync::oneshot;

const DARX_BUNDLES_DIR: &str = "./darx_bundles";

pub async fn run_server(
    socket_addr: SocketAddr,
    working_dir: PathBuf,
) -> Result<()> {
    let working_dir = fs::canonicalize(working_dir)
        .await
        .context("Failed to canonicalize working dir")?;
    let bundles_dir = working_dir.join(crate::DARX_BUNDLES_DIR);
    fs::create_dir_all(bundles_dir.as_path()).await?;

    #[cfg(debug_assertions)]
    dotenv().expect("Failed to load .env file");

    let worker_pool = WorkerPool::new();
    let db_pool = sqlx::MySqlPool::connect(
        env::var("DATABASE_URL")
            .expect("DATABASE_URL should be configured")
            .as_str(),
    )
    .await
    .context("Failed to connect database")?;

    init_deployments(bundles_dir.as_path(), &db_pool)
        .await
        .context("Failed to init deployments on startup")?;

    let server_state = Arc::new(ServerState {
        worker_pool,
        bundles_dir,
    });

    let app = Router::new()
        .route("/", get(|| async { "data plane healthy." }))
        .route("/invoke/*func_url", post(invoke_function))
        .route("/add_deployment", post(add_deployment))
        .with_state(server_state);

    tracing::info!("listen on {}", socket_addr);
    axum::Server::bind(&socket_addr)
        .serve(app.into_make_service())
        .await
        .context("Failed to start http server")?;
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

    let bundle_dir = find_bundle_dir(
        server_state.bundles_dir.as_path(),
        env_id.as_str(),
        deploy_seq,
    )
    .await?;
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

    match resp_rx.await.with_context(|| {
        format!(
            "failed to receive response from worker pool for function '{}'",
            func_url
        )
    }) {
        Ok(r) => match r {
            Ok(v) => Ok(Json(v)),
            Err(e) => Err(ApiError::Internal(e)),
        },
        Err(e) => Err(ApiError::Internal(e)),
    }
}

async fn add_deployment(
    State(server_state): State<Arc<ServerState>>,
    Json(req): Json<AddDeploymentReq>,
) -> Result<StatusCode, ApiError> {
    let deployment_route = DeploymentRoute {
        env_id: req.env_id.clone(),
        deploy_seq: req.deploy_seq,
        http_routes: req.http_routes,
    };
    add_bundle_files(
        req.env_id.as_str(),
        req.deploy_seq,
        server_state.bundles_dir.as_path(),
        req.bundles,
    )
    .await?;
    add_route(deployment_route);
    Ok(StatusCode::OK)
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

// #[derive(Deserialize)]
// struct ConnectDBRequest {
//     project_id: String,
//     db_type: DBType,
//     db_url: String,
// }

struct ServerState {
    worker_pool: WorkerPool,
    bundles_dir: PathBuf,
}

fn extract_env_id(domain: &str) -> Result<String> {
    let mut parts = domain.split('.');
    let env_id = parts.next().ok_or_else(|| {
        ApiError::DomainNotFound(format!("invalid domain: {}", domain))
    })?;
    Ok(env_id.to_string())
}
