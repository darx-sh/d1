mod route_builder;

use crate::route_builder::build_route;
use anyhow::{anyhow, Context, Result};
use axum::extract::{Path as AxumPath, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use darx_api::{
    add_deployment_url, deploy_bundle_url, unique_js_export, AddDeploymentReq,
    ApiError, Bundle, BundleRsp, DeployBundleReq, DeployBundleRsp, HttpRoute,
    PrepareDeployReq, PrepareDeployRsp, UpdateBundleStatus,
};
use dotenvy::dotenv;
use handlebars::Handlebars;
use nanoid::nanoid;
use serde::Serialize;
use serde_json::json;
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;

pub async fn run_server(socket_addr: SocketAddr) -> Result<()> {
    #[cfg(debug_assertions)]
    dotenv().expect("Failed to load .env file");

    let db_pool = sqlx::MySqlPool::connect(
        env::var("DATABASE_URL")
            .expect("DATABASE_URL should be configured")
            .as_str(),
    )
    .await
    .context("Failed to connect database")?;
    let server_state = Arc::new(ServerState { db_pool });

    let app = Router::new()
        .route("/", get(|| async { "control plane healthy." }))
        .route("/prepare_deploy", post(prepare_deploy))
        .route("/deploy_bundle/:env_id/:deploy_seq", post(deploy_bundle))
        .with_state(server_state);

    tracing::info!("listen on {}", socket_addr);
    axum::Server::bind(&socket_addr)
        .serve(app.into_make_service())
        .await
        .context("Failed to start control plane server")?;
    Ok(())
}

async fn prepare_deploy(
    State(server_state): State<Arc<ServerState>>,
    Json(req): Json<PrepareDeployReq>,
) -> Result<Json<PrepareDeployRsp>, ApiError> {
    let db_pool = &server_state.db_pool;
    let mut routes = vec![];
    for meta in req.metas.iter() {
        for js_export in meta.exports.iter() {
            let route =
                build_route(meta.entry_point.as_str(), js_export.as_str())?;
            routes.push(route);
        }
    }
    let txn = db_pool
        .begin()
        .await
        .context("Failed to start database transaction")?;

    let env = sqlx::query!(
        "SELECT next_deploy_seq FROM envs WHERE id = ? FOR UPDATE",
        req.env_id
    )
    .fetch_optional(db_pool)
    .await
    .context("Failed to find env")?
    .ok_or(ApiError::EnvNotFound(req.env_id.clone()))?;

    sqlx::query!(
        "UPDATE envs SET next_deploy_seq = next_deploy_seq + 1 WHERE id = ?",
        req.env_id
    )
    .execute(db_pool)
    .await
    .context("Failed to update envs table")?;

    let deploy_seq = env.next_deploy_seq + 1;

    // create new deploy
    // todo: retry to avoid id collision
    let deploy_id = new_nano_id();
    sqlx::query!(
        "INSERT INTO deploys (id, updated_at, tag, description, env_id, deploy_seq, bundle_repo, bundle_cnt) VALUES (?, CURRENT_TIMESTAMP(3), ?, ?, ?, ?, ?, ?)",
        deploy_id,
        req.tag,
        req.description,
        req.env_id,
        deploy_seq,
        "db",
        req.bundles.len() as i64,
    )
    .execute(db_pool)
    .await
    .context("Failed to insert into deploys table")?;

    // create new bundle
    let mut bundle_ids = vec![];
    for bundle in req.bundles.iter() {
        let bundle_id = new_nano_id();
        sqlx::query!(
            "INSERT INTO bundles (id, updated_at, bytes, deploy_id, fs_path) VALUES (?, CURRENT_TIMESTAMP(3), ?, ?, ?)",
            bundle_id,
            bundle.bytes,
            deploy_id,
            bundle.fs_path,
        )
        .execute(db_pool)
        .await
        .context("Failed to insert into bundles table")?;
        bundle_ids.push(bundle_id);
    }
    let registry_bundle = registry_code(&routes)?;
    sqlx::query!(
        "INSERT INTO bundles (id, updated_at, bytes, deploy_id, fs_path, upload_status, code) VALUES (?, CURRENT_TIMESTAMP(3), ?, ?, ?, ?, ?)",
        new_nano_id(),
        registry_bundle.len() as i64,
        deploy_id,
        "__registry.js",
        "success",
        registry_bundle,
    ).execute(db_pool).await.context("Failed to insert registry bundle")?;

    // create new route
    for route in routes.iter() {
        let route_id = new_nano_id();
        sqlx::query!(
            "INSERT INTO http_routes (id, updated_at, method, js_entry_point, js_export, deploy_id, http_path) VALUES (?, CURRENT_TIMESTAMP(3), ?, ?, ?, ?, ?)",
            route_id,
            "POST",
            route.js_entry_point,
            route.js_export,
            deploy_id,
            route.http_path
        ).execute(db_pool).await.context("Failed to insert into http_routes table")?;
    }
    txn.commit()
        .await
        .context("Failed to commit database transaction")?;
    // return data plan url to client
    let mut bundles = vec![];
    for (bundle, id) in req.bundles.iter().zip(bundle_ids.iter()) {
        let url = deploy_bundle_url(req.env_id.as_str(), deploy_seq);
        bundles.push(BundleRsp {
            id: id.to_string(),
            fs_path: bundle.fs_path.clone(),
            upload_url: url,
            upload_method: "POST".to_string(),
        });
    }
    Ok(Json(PrepareDeployRsp { deploy_id, bundles }))
}

async fn deploy_bundle(
    State(server_state): State<Arc<ServerState>>,
    AxumPath((env_id, deploy_seq)): AxumPath<(String, i32)>,
    Json(req): Json<DeployBundleReq>,
) -> Result<Json<DeployBundleRsp>, ApiError> {
    let pool = &server_state.db_pool;
    let txn = pool
        .begin()
        .await
        .context("Failed to start database transaction")?;
    let res = sqlx::query!(
        "UPDATE bundles SET code = ?, upload_status = ? WHERE id = ?",
        req.code,
        "success",
        req.id
    )
    .execute(pool)
    .await
    .context("Failed to update bundles table")?;

    if res.rows_affected() == 0 {
        return Err(ApiError::BundleNotFound(req.id));
    }

    sqlx::query!("UPDATE deploys SET bundle_upload_cnt = bundle_upload_cnt + 1 WHERE env_id = ? AND deploy_seq = ?", env_id, deploy_seq)
        .execute(pool)
        .await
        .context("Failed to update deploys table")?;

    let finished_deploy = sqlx::query!(
        "SELECT id, env_id, deploy_seq, bundle_repo FROM deploys WHERE env_id = ? AND deploy_seq = ? AND bundle_upload_cnt = bundle_cnt",
        env_id, deploy_seq
    )
        .fetch_optional(pool)
        .await
        .context("Failed to fetch deploy")?;
    txn.commit().await.context("Failed to commit transaction")?;

    if let Some(deploy) = finished_deploy {
        // load bundles
        let bundles = sqlx::query_as!(Bundle, "SELECT bundles.id as id, bundles.fs_path as fs_path, bundles.code as code FROM bundles INNER JOIN deploys ON bundles.deploy_id = deploys.id WHERE deploys.id = ? AND deploys.deploy_seq = ?", deploy.id, deploy.deploy_seq)
            .fetch_all(pool)
            .await
            .context("Failed to fetch bundles")?;

        // load http routes
        let http_routes = sqlx::query_as!(HttpRoute, "SELECT http_routes.method as method, http_routes.js_entry_point as js_entry_point, http_routes.js_export as js_export, http_routes.http_path as http_path FROM http_routes INNER JOIN deploys ON http_routes.deploy_id = deploys.id WHERE deploys.id = ? AND deploys.deploy_seq = ?", deploy.id, deploy.deploy_seq)
            .fetch_all(pool)
            .await
            .context("Failed to fetch http_routes")?;

        let req = AddDeploymentReq {
            env_id: deploy.env_id,
            deploy_seq: deploy.deploy_seq,
            bundle_repo: deploy.bundle_repo,
            bundles,
            http_routes,
        };
        let url = add_deployment_url();
        let rsp = reqwest::Client::new()
            .post(url)
            .json(&req)
            .send()
            .await
            .context("Failed to send add deployment request")?;
        if !rsp.status().is_success() {
            return Err(ApiError::Internal(anyhow!(
                "Failed to add deployment: {}",
                rsp.text().await.unwrap()
            )));
        }
    }
    Ok(Json(DeployBundleRsp {}))
}

const REGISTRY_TEMPLATE: &str = r#"
{{#each routes}}
import { {{js_export}} as {{ unique_export }} } from "./{{js_entry_point}}";
globalThis.{{unique_export}} = {{unique_export}};
{{/each}}
"#;

fn registry_code(routes: &Vec<HttpRoute>) -> Result<Vec<u8>> {
    #[derive(Serialize)]
    struct UniqueJsExport {
        js_entry_point: String,
        js_export: String,
        unique_export: String,
    }

    let mut unique_imports = vec![];
    for r in routes.iter() {
        let unique_export =
            unique_js_export(r.js_entry_point.as_str(), r.js_export.as_str());
        unique_imports.push(UniqueJsExport {
            js_entry_point: r.js_entry_point.clone(),
            js_export: r.js_export.clone(),
            unique_export,
        })
    }
    let mut reg = Handlebars::new();
    let code = reg.render_template(
        REGISTRY_TEMPLATE,
        &json!({ "routes": unique_imports }),
    )?;
    Ok(code.into_bytes())
}

struct ServerState {
    db_pool: sqlx::MySqlPool,
}

fn new_nano_id() -> String {
    let alphabet = "0123456789abcdefghijklmnopqrstuvwxyz";
    let chars = alphabet.chars().collect::<Vec<_>>();
    nanoid!(12, &chars)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_registry_code() -> Result<()> {
        let routes = vec![
            HttpRoute {
                http_path: "foo".to_string(),
                method: "POST".to_string(),
                js_entry_point: "foo.js".to_string(),
                js_export: "default".to_string(),
            },
            HttpRoute {
                http_path: "foo.foo".to_string(),
                method: "POST".to_string(),
                js_entry_point: "foo.js".to_string(),
                js_export: "foo".to_string(),
            },
        ];
        let code = registry_code(&routes)?;
        assert_eq!(
            r#"
import { default as foo_default } from "./foo.js";
globalThis.foo_default = foo_default;
import { foo as foo_foo } from "./foo.js";
globalThis.foo_foo = foo_foo;
"#,
            String::from_utf8(code)?
        );
        Ok(())
    }
}
