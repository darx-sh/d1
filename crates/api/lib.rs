use std::env;

use actix_web::http::header::ContentType;
use actix_web::http::StatusCode;
use actix_web::web::Json;
use actix_web::{HttpResponse, ResponseError};
use serde::{Deserialize, Serialize};
use serde_json::json;
use thiserror::Error;

/// Re-export darx_db types.
pub use darx_db::DBType;

pub mod deploy;

pub fn add_deployment_url() -> String {
    format!(
        "{}/add_deployment",
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Code {
    pub fs_path: String,
    pub content: String,
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
/// add_deployment
///
#[derive(Debug, Serialize, Deserialize)]
pub struct AddDeploymentReq {
    pub env_id: String,
    pub deploy_seq: i32,
    pub codes: Vec<Code>,
    pub http_routes: Vec<HttpRoute>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HttpRoute {
    pub http_path: String,
    pub method: String,
    /// `js_entry_point` is used to find the js file.
    pub js_entry_point: String,
    pub js_export: String,

    pub func_sig_version: i32,
    pub func_sig: FunctionSignatureV1,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FunctionSignatureV1 {
    pub export_name: String,
    pub param_names: Vec<String>,
}

/// [`unique_js_export`] returns a unique function name
pub fn unique_js_export(js_entry_point: &str, js_export: &str) -> String {
    let js_entry_point =
        js_entry_point.strip_suffix(".js").unwrap_or(js_entry_point);
    let js_entry_point =
        js_entry_point.strip_suffix(".ts").unwrap_or(js_entry_point);
    let js_entry_point = js_entry_point
        .strip_suffix(".mjs")
        .unwrap_or(js_entry_point);
    let new_entry = js_entry_point.split("/").collect::<Vec<_>>().join("_");
    format!("{}_{}", new_entry, js_export)
}

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Authorization failed")]
    Auth,
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Domain {0} not found")]
    DomainNotFound(String),
    #[error("Bundle {0} not found")]
    BundleNotFound(String),
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

            ApiError::DomainNotFound(_) => {
                HttpResponse::build(StatusCode::NOT_FOUND)
                    .insert_header(ContentType::plaintext())
                    .body(self.to_string())
            }

            ApiError::BundleNotFound(_) => {
                HttpResponse::build(StatusCode::NOT_FOUND)
                    .insert_header(ContentType::plaintext())
                    .body(self.to_string())
            }

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

            ApiError::TableNotFound(_) => {
                HttpResponse::build(StatusCode::NOT_FOUND)
                    .insert_header(ContentType::plaintext())
                    .body(self.to_string())
            }

            ApiError::Internal(_) => {
                HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                    .json(json!({"error": self.to_string()}))
            }

            ApiError::EnvNotFound(_) => {
                HttpResponse::build(StatusCode::NOT_FOUND)
                    .json(json!({"error": self.to_string()}))
            }

            ApiError::Timeout => {
                HttpResponse::build(StatusCode::REQUEST_TIMEOUT)
                    .json(json!({"error": self.to_string()}))
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    result: T,
}

pub type JsonApiResponse<T> = Json<ApiResponse<T>>;

pub const REGISTRY_FILE_NAME: &str = "__registry.js";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unique_js_export() {
        assert_eq!(unique_js_export("foo.js", "bar"), "foo_bar");
        assert_eq!(unique_js_export("foo/foo.js", "bar"), "foo_foo_bar");
    }
}
