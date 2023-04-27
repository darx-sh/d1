#![allow(dead_code)]

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod catalog;
mod expr;
mod list_entity;
mod row;

pub use list_entity::list_entity;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Authorization failed")]
    Auth,
    #[error("Table {0} not found")]
    TableNotFound(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::Auth => {
                (StatusCode::UNAUTHORIZED, format!("{}", self)).into_response()
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
