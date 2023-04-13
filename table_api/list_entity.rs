use crate::catalog::CatalogRef;
use crate::expr::Statement;
use crate::row::Row;
use crate::{ApiError, ApiResponse, JsonApiResponse};
use anyhow::Result;
use axum::extract::{Path, Query};
use axum::Json;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Serialize, Default)]
pub struct ListEntityResponse {
    data: Vec<Row>,
}

pub async fn list_entity(
    Query(params): Query<HashMap<String, String>>,
    Path(_table_name): Path<String>,
) -> Result<JsonApiResponse<ListEntityResponse>, ApiError> {
    let _params = QueryParams::from(params);
    Ok(Json(ApiResponse {
        result: Default::default(),
    }))
}

struct QueryParams {
    select: Option<String>,
    filter: Option<String>,
    order: Option<String>,
    limit: Option<String>,
    offset: Option<String>,
}

impl From<HashMap<String, String>> for QueryParams {
    fn from(params: HashMap<String, String>) -> Self {
        Self {
            select: params.get("select").map(|s| s.to_string()),
            filter: params.get("filter").map(|s| s.to_string()),
            order: params.get("order").map(|s| s.to_string()),
            limit: params.get("limit").map(|s| s.to_string()),
            offset: params.get("offset").map(|s| s.to_string()),
        }
    }
}

fn select_statement(
    catalog: CatalogRef,
    _query_param: QueryParams,
    table_name: String,
) -> Result<Statement> {
    if let Some(_table) = catalog.get_table(table_name.as_str()) {
        todo!()
    } else {
        return Err(ApiError::TableNotFound(table_name).into());
    }
}
