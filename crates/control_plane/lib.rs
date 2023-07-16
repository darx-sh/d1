mod esm_parser;
mod route_builder;

use crate::esm_parser::parse_module_export;
use crate::route_builder::build_route;
use anyhow::{anyhow, Context, Result};
use axum::extract::{Path as AxumPath, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use darx_api::{
    add_deployment_url, unique_js_export, AddDeploymentReq, ApiError, Bundle,
    Code, DeployCodeReq, DeployCodeRsp, HttpRoute, ListCodeRsp,
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
        .route("/deploy_code/:env_id", post(deploy_code))
        .route("/list_code/:env_id", get(list_code))
        .with_state(server_state);

    tracing::info!("listen on {}", socket_addr);
    axum::Server::bind(&socket_addr)
        .serve(app.into_make_service())
        .await
        .context("Failed to start control plane server")?;
    Ok(())
}

async fn deploy_code(
    State(server_state): State<Arc<ServerState>>,
    AxumPath(env_id): AxumPath<String>,
    Json(req): Json<DeployCodeReq>,
) -> Result<Json<DeployCodeRsp>, ApiError> {
    let mut http_routes = vec![];
    // extract js_exports
    for code in req.codes.iter() {
        let functions_dir = "functions/";
        if !code.fs_path.starts_with(functions_dir) {
            continue;
        }
        let js_exports =
            parse_module_export(code.fs_path.as_str(), code.content.as_str())?;
        for js_export in js_exports.iter() {
            // the http route should start without `functions`.
            let route = build_route(
                Some(functions_dir),
                code.fs_path.as_str(),
                js_export,
            )?;
            http_routes.push(route);
        }
    }
    // create new deploy
    let db_pool = &server_state.db_pool;
    let txn = db_pool
        .begin()
        .await
        .context("Failed to start database transaction")?;
    let env = sqlx::query!(
        "SELECT next_deploy_seq FROM envs WHERE id = ? FOR UPDATE",
        env_id
    )
    .fetch_optional(db_pool)
    .await
    .context("Failed to find env")?
    .ok_or(ApiError::EnvNotFound(env_id.clone()))?;

    sqlx::query!(
        "UPDATE envs SET next_deploy_seq = next_deploy_seq + 1 WHERE id = ?",
        env_id
    )
    .execute(db_pool)
    .await
    .context("Failed to update envs table")?;

    let deploy_seq = env.next_deploy_seq + 1;
    let deploy_id = new_nano_id();
    sqlx::query!(
        "INSERT INTO deploys (id, updated_at, tag, description, env_id, deploy_seq, bundle_repo, bundle_cnt, bundle_upload_cnt) VALUES (?, CURRENT_TIMESTAMP(3), ?, ?, ?, ?, ?, ?, ?)",
        deploy_id,
        req.tag,
        req.desc,
        env_id,
        deploy_seq,
        "db",
        req.codes.len() as i64,
        req.codes.len() as i64,
    )
        .execute(db_pool)
        .await
        .context("Failed to insert into deploys table")?;

    // create new bundles
    // create new bundle
    let mut bundles = vec![];
    for bundle in req.codes.iter() {
        let bundle_id = new_nano_id();
        sqlx::query!(
            "INSERT INTO bundles (id, updated_at, bytes, deploy_id, fs_path, code, upload_status) VALUES (?, CURRENT_TIMESTAMP(3), ?, ?, ?, ?, ?)",
            bundle_id,
            bundle.content.len() as i64,
            deploy_id,
            bundle.fs_path,
            bundle.content,
            "success",
        )
            .execute(db_pool)
            .await
            .context("Failed to insert into bundles table")?;
        bundles.push(Bundle {
            id: bundle_id,
            fs_path: bundle.fs_path.clone(),
            code: Some(bundle.content.clone().into_bytes()),
        });
    }

    let registry_bundle = registry_code(&http_routes)?;
    let registry_bundle_id = new_nano_id();
    sqlx::query!(
        "INSERT INTO bundles (id, updated_at, bytes, deploy_id, fs_path, upload_status, code) VALUES (?, CURRENT_TIMESTAMP(3), ?, ?, ?, ?, ?)",
        registry_bundle_id,
        registry_bundle.len() as i64,
        deploy_id,
        "__registry.js",
        "success",
        registry_bundle,
    ).execute(db_pool).await.context("Failed to insert registry bundle")?;
    bundles.push(Bundle {
        id: registry_bundle_id,
        fs_path: "__registry.js".to_string(),
        code: Some(registry_bundle),
    });
    // create new http_routes
    for route in http_routes.iter() {
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

    let req = AddDeploymentReq {
        env_id,
        deploy_seq,
        bundle_repo: "db".to_string(),
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
    Ok(Json(DeployCodeRsp {}))
}

async fn list_code(
    State(server_state): State<Arc<ServerState>>,
    AxumPath(env_id): AxumPath<String>,
) -> Result<Json<ListCodeRsp>, ApiError> {
    let db_pool = &server_state.db_pool;
    let bundle = sqlx::query!(
        "
    SELECT bundles.fs_path AS fs_path, bundles.code AS code
    FROM bundles INNER JOIN deploys ON bundles.deploy_id = deploys.id
    WHERE deploys.env_id = ? ORDER BY deploys.deploy_seq DESC LIMIT 1",
        env_id
    )
    .fetch_all(db_pool)
    .await
    .context("Failed to query bundles table")?;
    let mut codes = vec![];
    for b in bundle.iter() {
        let content = b.code.as_ref().unwrap();
        codes.push(Code {
            fs_path: b.fs_path.clone(),
            content: String::from_utf8(content.clone()).unwrap(),
        });
    }
    let rsp = ListCodeRsp { codes };
    Ok(Json(rsp))
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
    let reg = Handlebars::new();
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
