use crate::{Code, DeploySeq, HttpRoute};
use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use async_recursion::async_recursion;
use darx_db::TenantDBInfo;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};
use thiserror::Error;
use tokio::fs;
use tracing::info;

pub fn add_code_deploy_url() -> String {
  format!(
    "{}/add_code_deploy",
    env::var("DATA_PLANE_URL")
      .expect("DATA_PLANE_URL should be configured to add route"),
  )
}

pub fn add_plugin_deploy_url() -> String {
  format!(
    "{}/add_plugin_deploy",
    env::var("DATA_PLANE_URL")
      .expect("DATA_PLANE_URL should be configured to add route"),
  )
}

pub fn add_var_deploy_url() -> String {
  format!(
    "{}/add_var_deploy",
    env::var("DATA_PLANE_URL")
      .expect("DATA_PLANE_URL should be configured to add route"),
  )
}

pub fn add_tenant_db_url() -> String {
  format!(
    "{}/add_tenant_db",
    env::var("DATA_PLANE_URL")
      .expect("DATA_PLANE_URL should be configured to add route"),
  )
}

///
/// deploy_code
///
#[derive(Debug, Serialize, Deserialize)]
pub struct DeployCodeReq {
  pub tag: Option<String>,
  pub desc: Option<String>,
  pub codes: Vec<Code>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeployCodeRsp {
  pub http_routes: Vec<HttpRoute>,
}

///
/// deploy_vars
///
#[derive(Debug, Serialize, Deserialize)]
pub struct DeployVarReq {
  pub desc: Option<String>,
  pub vars: HashMap<String, String>,
}

///
/// deploy_plugin
///
#[derive(Serialize, Deserialize)]
pub struct DeployPluginReq {
  pub codes: Vec<Code>,
}

///
/// list code
///
#[derive(Serialize, Deserialize)]
pub struct ListCodeRsp {
  pub project: ProjectInfo,
  pub env: EnvInfo,
  pub codes: Vec<Code>,
  pub http_routes: Vec<HttpRoute>,
}

///
/// list api
///
#[derive(Serialize, Deserialize)]
pub struct ListApiRsp {
  pub http_routes: Vec<HttpRoute>,
}

#[derive(Serialize, Deserialize)]
pub struct ListProjectRsp {
  pub projects: Vec<ProjectInfo>,
}

#[derive(Serialize, Deserialize)]
pub struct ProjectInfo {
  pub id: String,
  pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct EnvInfo {
  pub id: String,
  pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct NewTenantProjectReq {
  pub org_id: String,
  pub project_name: String,
}

#[derive(Serialize, Deserialize)]
pub struct NewPluginProjectReq {
  pub org_id: String,
  pub plugin_name: String,
}

#[derive(Serialize, Deserialize)]
pub struct NewProjectRsp {
  pub project: ProjectInfo,
  pub env: EnvInfo,
}

///
///  control plane --> data plane api begins.
///
#[derive(Debug, Serialize, Deserialize)]
pub struct AddTenantDBReq {
  pub env_id: String,
  pub db_info: TenantDBInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddPluginDeployReq {
  pub name: String,
  pub env_id: String,
  pub deploy_seq: DeploySeq,
  pub codes: Vec<Code>,
  pub http_routes: Vec<HttpRoute>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddCodeDeployReq {
  pub env_id: String,
  pub deploy_seq: DeploySeq,
  pub codes: Vec<Code>,
  pub http_routes: Vec<HttpRoute>,
}
///
/// control plane --> data plane api ends.
///

#[derive(Debug, Serialize, Deserialize)]
pub struct AddVarDeployReq {
  pub env_id: String,
  pub deploy_seq: DeploySeq,
  pub vars: HashMap<String, String>,
}

#[derive(Error, Debug)]
pub enum ApiError {
  #[error("Authorization failed")]
  AuthError,
  #[error("IO error: {0}")]
  IoError(#[from] std::io::Error),
  #[error("Domain {0} not found")]
  DomainNotFound(String),
  #[error("Deploy {0} not found")]
  DeployNotFound(String),
  #[error("Function {0} not found")]
  FunctionNotFound(String),
  #[error("Function parameter error: {0}")]
  FunctionParameterError(String),
  #[error("js runtime error: {0:?}")]
  FunctionRuntimeError(anyhow::Error),
  #[error("parse error: {0}")]
  FunctionParseError(String),
  #[error("Table {0} not found")]
  TableNotFound(String),
  #[error("Internal error: {0:?}")]
  Internal(anyhow::Error),
  #[error("Environment {0} not found")]
  EnvNotFound(String),
  #[error("Project {0} not found")]
  ProjectNotFound(String),
  #[error("Invalid plugin url: {0}")]
  InvalidPluginUrl(String),
  #[error("function execution timeout")]
  Timeout,
}

impl From<anyhow::Error> for ApiError {
  fn from(err: anyhow::Error) -> Self {
    ApiError::Internal(err)
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse<T> {
  pub error: Error<T>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Error<T> {
  pub code: i32, // 3 digits of http status code + 2 digits of custom error code

  #[serde(rename = "type")]
  pub typ: Cow<'static, str>, // string representation of ApiError type

  pub message: String,

  #[serde(skip_serializing_if = "Option::is_none")]
  pub details: Option<T>,
}

impl ApiError {
  pub const fn error_code(&self) -> (StatusCode, i32) {
    match self {
      ApiError::AuthError => (StatusCode::UNAUTHORIZED, 40100),
      ApiError::IoError(_) => (StatusCode::INTERNAL_SERVER_ERROR, 50000),
      ApiError::DomainNotFound(_) => (StatusCode::NOT_FOUND, 40400),
      ApiError::DeployNotFound(_) => (StatusCode::NOT_FOUND, 40401),
      ApiError::FunctionNotFound(_) => (StatusCode::NOT_FOUND, 40402),
      ApiError::FunctionParameterError(_) => (StatusCode::BAD_REQUEST, 40000),
      ApiError::FunctionRuntimeError(_) => {
        (StatusCode::INTERNAL_SERVER_ERROR, 50001)
      }
      ApiError::FunctionParseError(_) => (StatusCode::BAD_REQUEST, 40001),
      ApiError::TableNotFound(_) => (StatusCode::NOT_FOUND, 40403),
      ApiError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, 50099),
      ApiError::EnvNotFound(_) => (StatusCode::NOT_FOUND, 40404),
      ApiError::ProjectNotFound(_) => (StatusCode::NOT_FOUND, 40405),
      ApiError::InvalidPluginUrl(_) => (StatusCode::BAD_REQUEST, 40002),
      ApiError::Timeout => (StatusCode::INTERNAL_SERVER_ERROR, 50002),
    }
  }
}

macro_rules! build_error_response {
  ($self:expr, $typ:expr) => {
    HttpResponse::build($self.error_code().0).json(ErrorResponse {
      error: Error::<()> {
        code: $self.error_code().1,
        typ: Cow::from($typ),
        message: $self.to_string(),
        details: None,
      },
    })
  };
}

impl ResponseError for ApiError {
  fn error_response(&self) -> HttpResponse {
    match self {
      ApiError::AuthError => build_error_response!(self, "AuthError"),
      ApiError::IoError(_) => build_error_response!(self, "IoError"),
      ApiError::DomainNotFound(_) => {
        build_error_response!(self, "DomainNotFound")
      }
      ApiError::DeployNotFound(_) => {
        build_error_response!(self, "DeployNotFound")
      }
      ApiError::FunctionNotFound(_) => {
        build_error_response!(self, "FunctionNotFound")
      }
      ApiError::FunctionParameterError(_) => {
        build_error_response!(self, "FunctionParameterError")
      }
      ApiError::FunctionRuntimeError(_) => {
        build_error_response!(self, "FunctionRuntimeError")
      }
      ApiError::FunctionParseError(_) => {
        build_error_response!(self, "FunctionParseError")
      }
      ApiError::TableNotFound(_) => {
        build_error_response!(self, "TableNotFound")
      }
      ApiError::Internal(_) => build_error_response!(self, "Internal"),
      ApiError::EnvNotFound(_) => build_error_response!(self, "EnvNotFound"),
      ApiError::ProjectNotFound(_) => {
        build_error_response!(self, "ProjectNotFound")
      }
      ApiError::InvalidPluginUrl(_) => {
        build_error_response!(self, "InvalidPluginUrl")
      }
      ApiError::Timeout => build_error_response!(self, "Timeout"),
    }
  }
}

pub async fn dir_to_deploy_code_req(
  dir: &Path,
) -> anyhow::Result<DeployCodeReq> {
  let codes = collect_code(dir).await?;
  let req = DeployCodeReq {
    tag: None,
    desc: None,
    codes,
  };
  Ok(req)
}

pub async fn dir_to_deploy_plugin_req(
  dir: &Path,
) -> anyhow::Result<DeployPluginReq> {
  let codes = collect_code(dir).await?;
  let req = DeployPluginReq { codes };
  Ok(req)
}

async fn collect_code(dir: &Path) -> anyhow::Result<Vec<Code>> {
  let mut file_list_path_vec = vec![];
  collect_js_file_list(&mut file_list_path_vec, dir).await?;
  let fs_path_str_vec = file_list_path_vec
    .iter()
    .map(|path| {
      path
        .strip_prefix(dir)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string()
    })
    .collect::<Vec<_>>();
  let mut codes = vec![];

  for (path, fs_path_str) in
    file_list_path_vec.iter().zip(fs_path_str_vec.iter())
  {
    if fs_path_str.starts_with("functions/") {
      let content = fs::read_to_string(path).await?;
      codes.push(Code {
        fs_path: fs_path_str.clone(),
        content,
      });
      info!("upload: {}", fs_path_str);
    } else {
      info!(
        "ignore code outside of functions directory: {}",
        fs_path_str
      );
    }
  }
  Ok(codes)
}

#[async_recursion]
async fn collect_js_file_list(
  file_list: &mut Vec<PathBuf>,
  cur_dir: &Path,
) -> anyhow::Result<()> {
  let mut entries = fs::read_dir(cur_dir).await?;
  while let Some(entry) = entries.next_entry().await? {
    let entry_path = entry.path();
    if entry_path.is_dir() {
      collect_js_file_list(file_list, entry_path.as_path()).await?;
    } else if let Some(ext) = entry_path.extension() {
      if ext == "ts" || ext == "js" {
        file_list.push(entry_path);
      }
    }
  }
  Ok(())
}
