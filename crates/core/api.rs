use crate::env_vars::var::Var;
use crate::{Code, DeploySeq, EnvKind, HttpRoute};
use actix_web::http::header::ContentType;
use actix_web::http::StatusCode;
use actix_web::web::Json;
use actix_web::{HttpResponse, ResponseError};
use async_recursion::async_recursion;
use serde::{Deserialize, Serialize};
use serde_json::json;
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

pub fn add_var_deploy_url() -> String {
  format!(
    "{}/add_var_deploy",
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
  pub vars: Vec<Var>,
}

///
/// deploy_plugin
///
#[derive(Serialize, Deserialize)]
pub struct DeployPluginReq {
  pub name: String,
  pub env_kind: EnvKind,
  pub codes: Vec<Code>,
  pub vars: Vec<Var>,
}

///
/// list code
///
#[derive(Serialize, Deserialize)]
pub struct ListCodeRsp {
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
pub struct NewProjectReq {
  pub org_id: String,
  pub project_name: String,
}

#[derive(Serialize, Deserialize)]
pub struct NewProjectRsp {
  pub project_id: String,
  pub env_id: String,
}

///
/// add_deployment: control plane --> data plane
///
#[derive(Debug, Serialize, Deserialize)]
pub struct AddCodeDeployReq {
  pub env_id: String,
  pub deploy_seq: DeploySeq,
  pub codes: Vec<Code>,
  pub http_routes: Vec<HttpRoute>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddVarDeployReq {
  pub env_id: String,
  pub deploy_seq: DeploySeq,
  pub vars: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
  result: T,
}

pub type JsonApiResponse<T> = Json<ApiResponse<T>>;

#[derive(Error, Debug)]
pub enum ApiError {
  #[error("Authorization failed")]
  Auth,
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
  #[error("Table {0} not found")]
  TableNotFound(String),
  #[error("Internal error: {0:?}")]
  Internal(anyhow::Error),
  #[error("Environment {0} not found")]
  EnvNotFound(String),
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

impl ResponseError for ApiError {
  fn error_response(&self) -> HttpResponse {
    match self {
      ApiError::Auth => HttpResponse::build(StatusCode::UNAUTHORIZED)
        .insert_header(ContentType::plaintext())
        .body(self.to_string()),

      ApiError::IoError(_) => {
        HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
          .insert_header(ContentType::plaintext())
          .body(self.to_string())
      }

      ApiError::DomainNotFound(_) => HttpResponse::build(StatusCode::NOT_FOUND)
        .insert_header(ContentType::plaintext())
        .body(self.to_string()),

      ApiError::DeployNotFound(_) => HttpResponse::build(StatusCode::NOT_FOUND)
        .insert_header(ContentType::plaintext())
        .body(self.to_string()),

      ApiError::FunctionNotFound(_) => {
        HttpResponse::build(StatusCode::NOT_FOUND)
          .insert_header(ContentType::plaintext())
          .body(self.to_string())
      }

      ApiError::FunctionParameterError(_) => {
        HttpResponse::build(StatusCode::BAD_REQUEST)
          .insert_header(ContentType::plaintext())
          .body(self.to_string())
      }

      ApiError::TableNotFound(_) => HttpResponse::build(StatusCode::NOT_FOUND)
        .insert_header(ContentType::plaintext())
        .body(self.to_string()),

      ApiError::Internal(_) => {
        HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
          .json(json!({"error": self.to_string()}))
      }

      ApiError::EnvNotFound(_) => HttpResponse::build(StatusCode::NOT_FOUND)
        .json(json!({"error": self.to_string()})),
      ApiError::InvalidPluginUrl(_) => {
        HttpResponse::build(StatusCode::NOT_FOUND)
          .json(json!({"error": self.to_string()}))
      }

      ApiError::Timeout => HttpResponse::build(StatusCode::REQUEST_TIMEOUT)
        .json(json!({"error": self.to_string()})),
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
  plugin_name: &str,
  dir: &Path,
) -> anyhow::Result<DeployPluginReq> {
  let codes = collect_code(dir).await?;
  let req = DeployPluginReq {
    name: plugin_name.to_string(),
    env_kind: EnvKind::Dev,
    codes,
    vars: vec![],
  };
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
