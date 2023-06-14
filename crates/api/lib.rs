use axum::http;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};

/// Re-export darx_db types.
pub use darx_db::{
    Bundle, DBType, DeploymentId, DeploymentStatus, DeploymentType, Migration,
};

use serde_json::json;
use thiserror::Error;

#[derive(Deserialize)]
pub struct CreatProjectRequest {
    // project_id should unique in the system.
    pub project_id: String,
    pub db_type: DBType,
    pub db_url: Option<String>,
}

#[derive(Deserialize)]
pub struct DeploySchemaRequest {
    pub migrations: Vec<Migration>,
}

#[derive(Serialize)]
pub struct DeploySchemaResponse {
    pub deployment_id: DeploymentId,
}

#[derive(Serialize, Deserialize)]
pub struct DeployFunctionsRequest {
    pub bundles: Vec<Bundle>,
    pub bundle_meta: serde_json::Value,
    pub description: Option<String>,
}

#[derive(Serialize)]
pub struct DeployFunctionsResponse {
    pub deployment_id: DeploymentId,
}

#[derive(Serialize)]
pub struct GetDeploymentResponse {
    pub deploy_type: DeploymentType,
    pub status: DeploymentStatus,
}

#[derive(Deserialize)]
pub struct RollbackFunctionsRequest {
    pub target_deployment_id: i64,
}

/// Rollback will create another deployment [`new_deployment_id`].
#[derive(Serialize)]
pub struct RollbackFunctionsResponse {
    pub new_deployment_id: i64,
}

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Authorization failed")]
    Auth,
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Function {0} not found")]
    FunctionNotFound(String),
    #[error("Function parameter error: {0}")]
    FunctionParameterError(String),
    #[error("Table {0} not found")]
    TableNotFound(String),
    #[error("Internal error")]
    Internal(anyhow::Error),
}

impl From<anyhow::Error> for ApiError {
    fn from(err: anyhow::Error) -> Self {
        ApiError::Internal(err)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::Auth => {
                (http::StatusCode::UNAUTHORIZED, format!("{}", self))
                    .into_response()
            }
            ApiError::IoError(e) => {
                (http::StatusCode::INTERNAL_SERVER_ERROR, format!("{:#}", e))
                    .into_response()
            }
            ApiError::FunctionNotFound(_) => {
                (http::StatusCode::NOT_FOUND, format!("{}", self))
                    .into_response()
            }
            ApiError::FunctionParameterError(_) => {
                (http::StatusCode::BAD_REQUEST, format!("{}", self))
                    .into_response()
            }
            ApiError::TableNotFound(_) => {
                (http::StatusCode::NOT_FOUND, format!("{}", self))
                    .into_response()
            }
            ApiError::Internal(e) => (
                http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("{:#}", e) })),
            )
                .into_response(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    result: T,
}

pub type JsonApiResponse<T> = Json<ApiResponse<T>>;
