use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Authorization failed")]
    Auth,
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
                (StatusCode::UNAUTHORIZED, format!("{}", self)).into_response()
            }
            ApiError::FunctionNotFound(_) => {
                (StatusCode::NOT_FOUND, format!("{}", self)).into_response()
            }
            ApiError::FunctionParameterError(_) => {
                (StatusCode::BAD_REQUEST, format!("{}", self)).into_response()
            }
            ApiError::TableNotFound(_) => {
                (StatusCode::NOT_FOUND, format!("{}", self)).into_response()
            }
            ApiError::Internal(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
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
