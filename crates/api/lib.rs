use anyhow::Result;
use axum::http;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use std::env;

/// Re-export darx_db types.
pub use darx_db::DBType;

use serde_json::json;
use thiserror::Error;

pub fn deploy_bundle_url(env_id: &str, deploy_seq: i32) -> String {
    format!(
        "{}/deploy_bundle/{}/{}",
        env::var("CONTROL_PLANE_URL")
            .expect("CONTROL_PLANE_URL should be configured"),
        env_id,
        deploy_seq,
    )
}

pub fn update_bundle_status_url(bundle_id: &str) -> String {
    format!(
        "{}/update_bundle_status/{}",
        env::var("CONTROL_PLANE_URL")
            .expect("CONTROL_PLANE_URL should be configured"),
        bundle_id,
    )
}

pub fn add_deployment_url() -> String {
    format!(
        "{}/add_deployment",
        env::var("DATA_PLANE_URL")
            .expect("DATA_PLANE_URL should be configured to add route"),
    )
}

#[derive(Serialize, Deserialize)]
pub struct PrepareDeployReq {
    pub env_id: String,
    pub tag: Option<String>,
    pub description: Option<String>,
    pub bundles: Vec<BundleReq>,
    pub metas: Vec<BundleMeta>,
}

#[derive(Serialize, Deserialize)]
pub struct PrepareDeployRsp {
    pub deploy_id: String,
    pub bundles: Vec<BundleRsp>,
}

#[derive(Serialize, Deserialize)]
pub struct BundleReq {
    pub fs_path: String,
    pub bytes: i64,
    pub checksum: String,
    pub checksum_type: String,
}

#[derive(Serialize, Deserialize)]
pub struct BundleRsp {
    pub id: String,
    pub fs_path: String,
    pub upload_url: String,
    pub upload_method: String,
}

#[derive(Serialize, Deserialize)]
pub struct BundleMeta {
    pub entry_point: String,
    pub exports: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct DeployBundleReq {
    pub id: String,
    pub fs_path: String,
    pub code: String,
}

#[derive(Serialize, Deserialize)]
pub struct DeployBundleRsp {}

#[derive(Serialize, Deserialize)]
pub struct UpdateBundleStatus {
    pub status: String,
}

#[derive(Serialize, Deserialize)]
pub struct AddDeploymentReq {
    pub env_id: String,
    pub deploy_seq: i32,
    pub bundle_repo: String,
    pub bundles: Vec<Bundle>,
    pub http_routes: Vec<HttpRoute>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Bundle {
    pub id: String,
    pub fs_path: String,
    pub code: Option<Vec<u8>>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct HttpRoute {
    pub http_path: String,
    pub method: String,
    /// `js_entry_point` is used to find the js file.
    pub js_entry_point: String,
    pub js_export: String,
}

#[derive(Deserialize)]
pub struct CreatProjectRequest {
    // project_id should unique in the system.
    pub project_id: String,
    pub db_type: DBType,
    pub db_url: Option<String>,
}

// #[derive(Deserialize)]
// pub struct DeploySchemaRequest {
//     pub migrations: Vec<Migration>,
// }
//
// #[derive(Serialize)]
// pub struct DeploySchemaResponse {
//     pub deployment_id: DeploymentId,
// }

//
// #[derive(Serialize, Deserialize)]
// pub struct DeployFunctionsRequest {
//     pub bundles: Vec<Bundle>,
//     pub bundle_meta: serde_json::Value,
//     pub description: Option<String>,
// }
//
// #[derive(Serialize)]
// pub struct DeployFunctionsResponse {
//     pub deployment_id: DeploymentId,
// }
//
// #[derive(Serialize)]
// pub struct GetDeploymentResponse {
//     pub deploy_type: DeploymentType,
//     pub status: DeploymentStatus,
// }
//
// #[derive(Deserialize)]
// pub struct RollbackFunctionsRequest {
//     pub target_deployment_id: i64,
// }
//
// /// Rollback will create another deployment [`new_deployment_id`].
// #[derive(Serialize)]
// pub struct RollbackFunctionsResponse {
//     pub new_deployment_id: i64,
// }

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
    #[error("Internal error")]
    Internal(anyhow::Error),
    #[error("Environment {0} not found")]
    EnvNotFound(String),
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
            ApiError::DomainNotFound(e) => {
                (http::StatusCode::NOT_FOUND, format!("{}", e)).into_response()
            }
            ApiError::BundleNotFound(e) => {
                (http::StatusCode::NOT_FOUND, format!("{}", e)).into_response()
            }
            ApiError::FunctionNotFound(e) => (
                http::StatusCode::NOT_FOUND,
                format!("function not found: {}", e),
            )
                .into_response(),
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
            ApiError::EnvNotFound(e) => (
                http::StatusCode::NOT_FOUND,
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

// pub async fn build_routes(meta_file: &str) -> Result<Vec<Route>> {
//     let mut file = File::open(meta_file).await?;
//     let mut buf = String::new();
//     file.read_to_string(&mut buf).await?;
//     let meta: serde_json::Value = serde_json::from_str(&buf)?;
//     let outputs = meta
//         .get("outputs")
//         .ok_or_else(|| anyhow!("No outputs found"))?
//         .as_object()
//         .ok_or_else(|| anyhow!("Outputs is not an object"))?;
//
//     let mut routes = vec![];
//     for (_, output) in outputs.iter() {
//         let output = output
//             .as_object()
//             .ok_or_else(|| anyhow!("Output is not an object"))?;
//         let nbytes = output
//             .get("bytes")
//             .ok_or_else(|| anyhow!("bytes not found"))?
//             .as_i64()
//             .ok_or_else(|| anyhow!("bytes is not a i64"))?;
//
//         if nbytes == 0 {
//             continue;
//         }
//
//         let entry_point = output
//             .get("entryPoint")
//             .ok_or_else(|| anyhow!("entryPoint not found"))?
//             .as_str()
//             .ok_or_else(|| anyhow!("entryPoint is not a string"))?
//             .to_string();
//
//         let exports = output
//             .get("exports")
//             .ok_or_else(|| anyhow!("exports not found"))?
//             .as_array()
//             .ok_or_else(|| anyhow!("exports is not an array"))?
//             .iter()
//             .map(|export| {
//                 export
//                     .as_str()
//                     .ok_or_else(|| anyhow!("export is not a string"))
//                     .map(|s| s.to_string())
//             })
//             .collect::<Result<Vec<_>>>()?;
//         for export in exports.iter() {
//             let http_path = build_path(&entry_point, &export)?;
//             routes.push(Route {
//                 http_path,
//                 js_entry_point: entry_point.clone(),
//                 js_export: export.clone(),
//             })
//         }
//     }
//     routes.sort_by(|a, b| a.http_path.cmp(&b.http_path));
//     Ok(routes)
// }
