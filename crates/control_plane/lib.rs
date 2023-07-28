use actix_cors::Cors;
use actix_web::dev::Server;
use actix_web::web::{get, post, Data, Json, Path};
use actix_web::{App, HttpServer};
use anyhow::{anyhow, Context, Result};
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing_actix_web::TracingLogger;

use darx_core::api::{
    add_deployment_url, AddDeploymentReq, ApiError, DeployCodeReq,
    DeployCodeRsp, ListCodeRsp,
};
use darx_core::deploy::control;

pub async fn run_server(socket_addr: SocketAddr) -> Result<Server> {
    let db_pool = sqlx::MySqlPool::connect(
        env::var("DATABASE_URL")
            .expect("DATABASE_URL should be configured")
            .as_str(),
    )
    .await
    .context("Failed to connect database")?;

    let server_state = Arc::new(ServerState { db_pool });

    tracing::info!("listen on {}", socket_addr);

    Ok(HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_method()
            .allow_any_header()
            .allow_any_origin();

        App::new()
            .wrap(TracingLogger::default())
            .wrap(cors)
            .app_data(Data::new(server_state.clone()))
            .route("/", get().to(|| async { "control plane healthy." }))
            .route("/deploy_code/{env_id}", post().to(deploy_code))
            .route("/list_code/{env_id}", get().to(list_code))
    })
    .bind(&socket_addr)?
    .run())
}

async fn deploy_code(
    server_state: Data<Arc<ServerState>>,
    env_id: Path<String>,
    req: Json<DeployCodeReq>,
) -> Result<Json<DeployCodeRsp>, ApiError> {
    let txn = server_state
        .db_pool
        .begin()
        .await
        .context("Failed to start transaction")?;
    let (deploy_seq, codes, http_routes, txn) = control::deploy_code(
        txn,
        env_id.as_str(),
        &req.codes,
        &req.tag,
        &req.desc,
    )
    .await?;

    let req = AddDeploymentReq {
        env_id: env_id.to_string(),
        deploy_seq,
        codes,
        http_routes: http_routes.clone(),
    };
    let url = add_deployment_url();
    let rsp = reqwest::Client::new()
        .post(url)
        .json(&req)
        .send()
        .await
        .context("Failed to send add deployment request")?;
    if !rsp.status().is_success() {
        return Err(ApiError::Internal(anyhow!(
            "Failed to add deployment: {}",
            rsp.text().await.unwrap()
        )));
    }

    txn.commit()
        .await
        .context("Failed to commit transaction when deploy_code")?;
    Ok(Json(DeployCodeRsp { http_routes }))
}

async fn list_code(
    server_state: Data<Arc<ServerState>>,
    env_id: Path<String>,
) -> Result<Json<ListCodeRsp>, ApiError> {
    let db_pool = &server_state.db_pool;
    let (codes, http_routes) =
        control::list_code(db_pool, env_id.as_str()).await?;
    Ok(Json(ListCodeRsp { codes, http_routes }))
}

struct ServerState {
    db_pool: sqlx::MySqlPool,
}
