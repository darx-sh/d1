use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
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
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    result: T,
}

pub type JsonApiResponse<T> = Json<ApiResponse<T>>;
