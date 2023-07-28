use actix_cors::Cors;
use actix_web::dev::{ConnectionInfo, Server};
use actix_web::web::{get, post, Data, Json, Path};
use actix_web::{
    App, HttpRequest, HttpResponse, HttpResponseBuilder, HttpServer,
};
use anyhow::{Context, Result};
use darx_core::deploy::data;
use darx_core::{api::AddDeploymentReq, api::ApiError};
use serde_json;
use std::env;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tracing::info;
use tracing_actix_web::TracingLogger;

const DARX_BUNDLES_DIR: &str = "./darx_bundles";

struct ServerState {
    bundles_dir: PathBuf,
}

pub async fn run_server(
    socket_addr: SocketAddr,
    working_dir: PathBuf,
) -> Result<Server> {
    fs::create_dir_all(working_dir.as_path())
        .await
        .context("Failed to create working directory")?;
    let working_dir = fs::canonicalize(working_dir)
        .await
        .context("Failed to canonicalize working dir")?;
    let bundles_dir = working_dir.join(DARX_BUNDLES_DIR);
    fs::create_dir_all(bundles_dir.as_path()).await?;

    let db_pool = sqlx::MySqlPool::connect(
        env::var("DATABASE_URL")
            .expect("DATABASE_URL should be configured")
            .as_str(),
    )
    .await
    .context("Failed to connect database")?;

    data::init_deployments(bundles_dir.as_path(), &db_pool)
        .await
        .context("Failed to init deployments on startup")?;

    info!("listen on {}", socket_addr);

    let server_state = Arc::new(ServerState { bundles_dir });
    Ok(HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_method()
            .allow_any_header()
            .allow_any_origin();
        App::new()
            .wrap(TracingLogger::default())
            .wrap(cors)
            .app_data(Data::new(server_state.clone()))
            .route("/", get().to(|| async { "data plane healthy." }))
            .route("/invoke/{func_url}", post().to(invoke_function))
            .route("/add_deployment", post().to(add_deployment))
    })
    .bind(&socket_addr)?
    .run())
}

async fn invoke_function(
    server_state: Data<Arc<ServerState>>,
    conn: ConnectionInfo,
    func_url: Path<String>,
    http_req: HttpRequest,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let darx_env = env::var("DARX_ENV").expect("DARX_ENV should be configured");
    let host = if darx_env != "production" {
        // dev environment use a special Darx-Dev-Host header to specify the host.
        http_req
            .headers()
            .get("Darx-Dev-Host")
            .map_or("".to_string(), |v| {
                v.to_str()
                    .context(
                        "Darx-Dev-Host contains non visible ascii characters",
                    )
                    .unwrap()
                    .to_string()
            })
    } else {
        conn.host().to_string()
    };
    let env_id = extract_env_id(host.as_str())?;
    let func_url = func_url.into_inner();

    let (deploy_seq, route) =
        data::match_route(env_id.as_str(), func_url.as_str(), "POST").ok_or(
            ApiError::FunctionNotFound(format!(
                "host: {}, env_id: {}",
                host, &env_id
            )),
        )?;
    let ret = data::invoke_function0(
        &server_state.bundles_dir,
        &env_id,
        deploy_seq,
        req,
        &route.js_entry_point,
        &route.js_export,
        &route.func_sig.param_names,
    )
    .await?;
    Ok(Json(ret))
}

async fn add_deployment(
    server_state: Data<Arc<ServerState>>,
    Json(req): Json<AddDeploymentReq>,
) -> Result<HttpResponseBuilder, ApiError> {
    data::add_deployment(
        server_state.bundles_dir.as_path(),
        req.env_id.as_str(),
        req.deploy_seq,
        &req.codes,
        &req.http_routes,
    )
    .await?;
    Ok(HttpResponse::Ok())
}

fn extract_env_id(domain: &str) -> Result<String> {
    let mut parts = domain.split('.');
    let env_id = parts.next().ok_or_else(|| {
        ApiError::DomainNotFound(format!("invalid domain: {}", domain))
    })?;
    Ok(env_id.to_string())
}
