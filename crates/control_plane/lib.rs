use actix_cors::Cors;
use actix_web::dev::Server;
use actix_web::web::{get, post, Data, Json, Path};
use actix_web::{App, HttpResponse, HttpResponseBuilder, HttpServer};
use anyhow::{anyhow, Context, Result};
use std::env;
use std::net::SocketAddr;
use tracing_actix_web::TracingLogger;

use darx_core::api::{
  add_code_deploy_url, add_plugin_deploy_url, add_tenant_db_url,
  add_var_deploy_url, AddCodeDeployReq, AddPluginDeployReq, AddTenantDBReq,
  AddVarDeployReq, ApiError, DeployCodeReq, DeployCodeRsp, DeployPluginReq,
  DeployVarReq, EnvInfo, ListApiRsp, ListCodeRsp, ListProjectRsp,
  NewPluginProjectReq, NewProjectRsp, NewTenantProjectReq, ProjectInfo,
};
use darx_core::code::control;
use darx_core::plugin::plugin_env_id;
use darx_core::Project;

pub async fn run_server(socket_addr: SocketAddr) -> Result<Server> {
  let db_pool = sqlx::MySqlPool::connect(
    env::var("DATABASE_URL")
      .expect("DATABASE_URL should be configured")
      .as_str(),
  )
  .await
  .context("Failed to connect database")?;

  let server_state = ServerState { db_pool };
  tracing::info!("listen on {}", socket_addr);

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
        .route("/", get().to(|| async { "control plane healthy." }))
        .route("/list_project/{org_id}", get().to(list_project))
        .route("/new_tenant_project", post().to(new_tenant_project))
        .route("/new_plugin_project", post().to(new_plugin_project))
        .route("/load_env/{project_id}", get().to(load_env))
        .route("/deploy_code/{env_id}", post().to(deploy_code))
        .route("/deploy_var/{env_id}", post().to(deploy_var))
        .route("/list_api/{env_id}", get().to(list_api))
        .route("/deploy_plugin/{plugin_name}", post().to(deploy_plugin))
    })
    .bind(&socket_addr)?
    .run(),
  )
}

async fn deploy_code(
  server_state: Data<ServerState>,
  env_id: Path<String>,
  req: Json<DeployCodeReq>,
) -> Result<Json<DeployCodeRsp>, ApiError> {
  let txn = server_state
    .db_pool
    .begin()
    .await
    .context("Failed to start transaction")?;
  let (deploy_seq, codes, http_routes, txn) =
    control::deploy_code(txn, env_id.as_str(), &req.codes, &req.tag, &req.desc)
      .await?;

  let req = AddCodeDeployReq {
    env_id: env_id.to_string(),
    deploy_seq,
    codes,
    http_routes: http_routes.clone(),
  };
  let url = add_code_deploy_url();
  let rsp = reqwest::Client::new()
    .post(url)
    .json(&req)
    .send()
    .await
    .context("Failed to send add_code_deploy request")?;
  if !rsp.status().is_success() {
    return Err(ApiError::Internal(anyhow!(
      "Failed to add deployment: {}",
      rsp.text().await.unwrap()
    )));
  }
  txn
    .commit()
    .await
    .context("Failed to commit transaction when deploy_code")?;
  Ok(Json(DeployCodeRsp { http_routes }))
}

async fn load_env(
  server_state: Data<ServerState>,
  proj_id: Path<String>,
) -> Result<Json<ListCodeRsp>, ApiError> {
  let db_pool = &server_state.db_pool;
  let (codes, http_routes, project, env) =
    control::load_env(db_pool, proj_id.as_str()).await?;
  Ok(Json(ListCodeRsp {
    codes,
    http_routes,
    project,
    env,
  }))
}

async fn deploy_var(
  server_state: Data<ServerState>,
  env_id: Path<String>,
  req: Json<DeployVarReq>,
) -> Result<HttpResponseBuilder, ApiError> {
  let txn = server_state
    .db_pool
    .begin()
    .await
    .context("Failed to start transaction")?;

  let (deploy_seq, vars, txn) =
    control::deploy_var(txn, env_id.as_str(), &req.vars, &req.desc).await?;

  let req = AddVarDeployReq {
    env_id: env_id.to_string(),
    deploy_seq,
    vars,
  };
  let url = add_var_deploy_url();
  let rsp = reqwest::Client::new()
    .post(url)
    .json(&req)
    .send()
    .await
    .context("Failed to send add_var_deploy request")?;
  if !rsp.status().is_success() {
    return Err(ApiError::Internal(anyhow!(
      "Failed to add var deploy: {}",
      rsp.text().await.unwrap()
    )));
  }
  txn
    .commit()
    .await
    .context("Failed to commit transaction when deploy_var")?;
  Ok(HttpResponse::Ok())
}

async fn list_api(
  server_state: Data<ServerState>,
  env_id: Path<String>,
) -> Result<Json<ListApiRsp>, ApiError> {
  let db_pool = &server_state.db_pool;
  let http_routes = control::list_api(db_pool, env_id.as_str()).await?;
  Ok(Json(ListApiRsp { http_routes }))
}

async fn deploy_plugin(
  server_state: Data<ServerState>,
  plugin_name: Path<String>,
  req: Json<DeployPluginReq>,
) -> Result<HttpResponseBuilder, ApiError> {
  let txn = server_state
    .db_pool
    .begin()
    .await
    .context("Failed to start transaction")?;
  let env_id = plugin_env_id(plugin_name.as_str());
  let (deploy_seq, codes, http_routes, txn) =
    control::deploy_code(txn, env_id.as_str(), &req.codes, &None, &None)
      .await?;
  let req = AddPluginDeployReq {
    name: plugin_name.clone(),
    env_id: env_id.to_string(),
    deploy_seq,
    codes,
    http_routes: http_routes.clone(),
  };
  let url = add_plugin_deploy_url();
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

  txn
    .commit()
    .await
    .context("Failed to commit transaction when deploy_code")?;

  Ok(HttpResponse::Ok())
}

async fn list_project(
  server_state: Data<ServerState>,
  org_id: Path<String>,
) -> Result<Json<ListProjectRsp>, ApiError> {
  let db_pool = &server_state.db_pool;
  let projects = Project::list_proj_info(org_id.as_str(), db_pool).await?;
  Ok(Json(ListProjectRsp { projects }))
}

async fn new_tenant_project(
  server_state: Data<ServerState>,
  req: Json<NewTenantProjectReq>,
) -> Result<Json<NewProjectRsp>, ApiError> {
  let db_pool = &server_state.db_pool;
  let project =
    Project::new_tenant_proj(req.org_id.as_str(), req.project_name.as_str());
  let txn = db_pool.begin().await.context("Failed to start txn")?;
  let txn = project.save(txn).await?;
  let req = AddTenantDBReq {
    env_id: project.env_id().to_string(),
    db_info: project.db_info().as_ref().unwrap().clone(),
  };
  let url = add_tenant_db_url();
  let rsp = reqwest::Client::new()
    .post(url)
    .json(&req)
    .send()
    .await
    .context("Failed to send add_tenant_db request")?;

  if !rsp.status().is_success() {
    return Err(ApiError::Internal(anyhow!(
      "Failed to add tenant db: {}",
      rsp.text().await.unwrap()
    )));
  }

  txn.commit().await.context("Failed to commit txn")?;

  tracing::info!("tenant db added: {:?}", project.db_info());

  Ok(Json(NewProjectRsp {
    project: ProjectInfo {
      id: project.id().to_string(),
      name: project.name().to_string(),
    },
    env: EnvInfo {
      id: project.env_id().to_string(),
      name: project.env_name().to_string(),
    },
  }))
}

async fn new_plugin_project(
  server_state: Data<ServerState>,
  req: Json<NewPluginProjectReq>,
) -> Result<Json<NewProjectRsp>, ApiError> {
  let db_pool = &server_state.db_pool;
  let project =
    Project::new_plugin_proj(req.org_id.as_str(), req.plugin_name.as_str());
  project
    .create_if_not_exist(&db_pool, req.plugin_name.as_str())
    .await?;
  Ok(Json(NewProjectRsp {
    project: ProjectInfo {
      id: project.id().to_string(),
      name: project.name().to_string(),
    },
    env: {
      EnvInfo {
        id: project.env_id().to_string(),
        name: project.env_name().to_string(),
      }
    },
  }))
}

#[derive(Clone)]
struct ServerState {
  db_pool: sqlx::MySqlPool,
}
