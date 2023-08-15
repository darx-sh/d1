use actix_cors::Cors;
use actix_web::dev::{ConnectionInfo, Server};
use actix_web::web::{get, post, scope, Data, Json, Path};
use actix_web::{
  App, HttpRequest, HttpResponse, HttpResponseBuilder, HttpServer,
};
use anyhow::{Context, Result};
use darx_core::api::{
  AddColumnReq, CreateTableReq, DropColumnReq, DropTableReq, RenameColumnReq,
};
use darx_core::tenants;
use darx_core::{api::AddDeploymentReq, api::ApiError};
use darx_db::MySqlTenantPool;
use serde_json;
use sqlx::MySqlPool;
use std::env;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tracing::info;
use tracing_actix_web::TracingLogger;

const DARX_ENVS_DIR: &str = "./darx_envs";

struct ServerState {
  envs_dir: PathBuf,
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
  let envs_dir = working_dir.join(DARX_ENVS_DIR);
  fs::create_dir_all(envs_dir.as_path()).await?;

  let db_pool = sqlx::MySqlPool::connect(
    env::var("DATABASE_URL")
      .expect("DATABASE_URL should be configured")
      .as_str(),
  )
  .await
  .context("Failed to connect database")?;

  tenants::init_deployments(envs_dir.as_path(), &db_pool)
    .await
    .context("Failed to init deployments on startup")?;

  info!("listen on {}", socket_addr);

  let server_state = Arc::new(ServerState { envs_dir });
  Ok(
    HttpServer::new(move || {
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
        .service(
          scope("/schema")
            .route("/create_table", post().to(create_table))
            .route("/drop_table", post().to(drop_table))
            .route("/add_column", post().to(add_column))
            .route("/drop_column", post().to(drop_column))
            .route("/rename_column", post().to(rename_column)),
        )
    })
    .bind(&socket_addr)?
    .run(),
  )
}

async fn invoke_function(
  server_state: Data<Arc<ServerState>>,
  conn: ConnectionInfo,
  func_url: Path<String>,
  http_req: HttpRequest,
  Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, ApiError> {
  let host = conn.host();
  let env_id = try_extract_env_id(host, &http_req)?;
  let func_url = func_url.into_inner();

  let (env_id, deploy_seq, route) =
    tenants::match_route(env_id.as_str(), func_url.as_str(), "POST").ok_or(
      ApiError::FunctionNotFound(format!(
        "host: {}, env_id: {}",
        host, &env_id
      )),
    )?;
  let ret = tenants::invoke_function(
    &server_state.envs_dir,
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
  tenants::add_deployment(
    server_state.envs_dir.as_path(),
    req.env_id.as_str(),
    req.deploy_seq,
    &req.codes,
    &req.http_routes,
  )
  .await?;
  Ok(HttpResponse::Ok())
}

async fn create_table(
  conn: ConnectionInfo,
  http_req: HttpRequest,
  Json(req): Json<CreateTableReq>,
) -> Result<HttpResponseBuilder, ApiError> {
  let pool = get_tenant_conn_pool(&conn, &http_req).await?;
  tenants::create_table(&pool, &req).await?;
  Ok(HttpResponse::Ok())
}

async fn drop_table(
  conn: ConnectionInfo,
  http_req: HttpRequest,
  Json(req): Json<DropTableReq>,
) -> Result<HttpResponseBuilder, ApiError> {
  let pool = get_tenant_conn_pool(&conn, &http_req).await?;
  tenants::drop_table(&pool, &req).await?;
  Ok(HttpResponse::Ok())
}

async fn add_column(
  conn: ConnectionInfo,
  http_req: HttpRequest,
  Json(req): Json<AddColumnReq>,
) -> Result<HttpResponseBuilder, ApiError> {
  let pool = get_tenant_conn_pool(&conn, &http_req).await?;
  tenants::add_column(&pool, &req).await?;
  Ok(HttpResponse::Ok())
}

async fn drop_column(
  conn: ConnectionInfo,
  http_req: HttpRequest,
  Json(req): Json<DropColumnReq>,
) -> Result<HttpResponseBuilder, ApiError> {
  let pool = get_tenant_conn_pool(&conn, &http_req).await?;
  tenants::drop_column(&pool, &req).await?;
  Ok(HttpResponse::Ok())
}

async fn rename_column(
  conn: ConnectionInfo,
  http_req: HttpRequest,
  Json(req): Json<RenameColumnReq>,
) -> Result<HttpResponseBuilder, ApiError> {
  let pool = get_tenant_conn_pool(&conn, &http_req).await?;
  tenants::rename_column(&pool, &req).await?;
  Ok(HttpResponse::Ok())
}

async fn get_tenant_conn_pool(
  conn: &ConnectionInfo,
  http_req: &HttpRequest,
) -> Result<MySqlPool> {
  let host = conn.host();
  let env_id = try_extract_env_id(host, &http_req)?;
  let pool = darx_db::get_tenant_pool(env_id.as_str()).await?;
  let pool = pool.as_any().downcast_ref::<MySqlTenantPool>().unwrap();
  Ok(pool.inner().clone())
}

fn try_extract_env_id(host: &str, http_req: &HttpRequest) -> Result<String> {
  let darx_env = env::var("DARX_ENV").expect("DARX_ENV should be configured");
  let host = if darx_env != "production" {
    // dev environment use a special Darx-Dev-Host header to specify the host.
    http_req
      .headers()
      .get("Darx-Dev-Host")
      .map_or("".to_string(), |v| {
        v.to_str()
          .context("Darx-Dev-Host contains non visible ascii characters")
          .unwrap()
          .to_string()
      })
  } else {
    host.to_string()
  };
  let env_id = env_id_from_domain(host.as_str())?;
  Ok(env_id)
}

fn env_id_from_domain(domain: &str) -> Result<String> {
  let mut parts = domain.split('.');
  let env_id = parts.next().ok_or_else(|| {
    ApiError::DomainNotFound(format!("invalid domain: {}", domain))
  })?;
  Ok(env_id.to_string())
}
