use crate::api_mgr::ApiError;
use anyhow::Result;
use axum::extract::{Path, Query};
use axum::routing::get;
use axum::{Json, Router};
use std::collections::HashMap;

pub async fn run_server() -> Result<()> {
    let app = Router::new().route(
        "/api/f/:function_name",
        get(invoke_func_get).post(invoke_func_post),
    );
    axum::Server::bind(&"127.0.0.1:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await?;
    Ok(())
}

async fn invoke_func_get(
    Query(params): Query<HashMap<String, String>>,
    Path(func_name): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    Ok(Json(serde_json::json!("hi")))
}

async fn invoke_func_post() {}
