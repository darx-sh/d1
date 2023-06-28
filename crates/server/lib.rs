use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::json;

mod api_builder;
mod control_plane;
mod server;
mod worker;

pub use server::run_server;

// todo: move to config
pub const DARX_SERVER_WORKING_DIR: &str = "./tmp/darx_bundles";
