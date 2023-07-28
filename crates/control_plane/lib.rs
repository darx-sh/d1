use std::env;
use std::net::SocketAddr;
use std::sync::Arc;

use actix_cors::Cors;
use actix_web::dev::Server;
use actix_web::web::{get, post, Data, Json, Path};
use actix_web::{App, HttpServer};
use anyhow::{anyhow, Context, Result};
use handlebars::Handlebars;
use nanoid::nanoid;
use serde::Serialize;
use serde_json::json;
use tracing_actix_web::TracingLogger;

use darx_core::{
    add_deployment_url, unique_js_export, AddDeploymentReq, ApiError, Code,
    DeployCodeReq, DeployCodeRsp, HttpRoute, ListCodeRsp, REGISTRY_FILE_NAME,
};

use crate::esm_parser::parse_module_export;
use crate::route_builder::build_route;

mod esm_parser;
mod route_builder;

pub async fn run_server(socket_addr: SocketAddr) -> Result<Server> {
    let db_pool = sqlx::MySqlPool::connect(
        env::var("DATABASE_URL")
            .expect("DATABASE_URL should be configured")
            .as_str(),
    )
    .await
    .context("Failed to connect database")?;

    let server_state = Arc::new(ServerState { db_pool });

    tracing::info!("listen on {}", socket_addr);

    Ok(HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_method()
            .allow_any_header()
            .allow_any_origin();

        App::new()
            .wrap(TracingLogger::default())
            .wrap(cors)
            .app_data(Data::new(server_state.clone()))
            .route("/", get().to(|| async { "control plane healthy." }))
            .route("/deploy_code/{env_id}", post().to(deploy_code))
            .route("/list_code/{env_id}", get().to(list_code))
    })
    .bind(&socket_addr)?
    .run())
}

async fn deploy_code(
    server_state: Data<Arc<ServerState>>,
    env_id: Path<String>,
    req: Json<DeployCodeReq>,
) -> Result<Json<DeployCodeRsp>, ApiError> {
    let env_id = env_id.into_inner();
    let mut http_routes = vec![];
    // extract js_exports
    for code in req.codes.iter() {
        let functions_dir = "functions/";
        if !code.fs_path.starts_with(functions_dir) {
            tracing::warn!(
                env = env_id,
                "code should starts with 'functions' {}",
                code.fs_path.as_str(),
            );
            continue;
        }
        let content = code.content.clone();
        let fn_sigs =
            parse_module_export(code.fs_path.as_str(), content.as_str())?;
        for sig in fn_sigs.iter() {
            // the http route should start without `functions`.
            let route =
                build_route(Some(functions_dir), code.fs_path.as_str(), sig)?;
            http_routes.push(route);
        }
    }
    // create new deploy
    let db_pool = &server_state.db_pool;
    let mut txn = db_pool
        .begin()
        .await
        .context("Failed to start database transaction when deploy_code")?;
    let env = sqlx::query!(
        "SELECT next_deploy_seq FROM envs WHERE id = ? FOR UPDATE",
        env_id
    )
    .fetch_optional(&mut txn)
    .await
    .context("Failed to find env")?
    .ok_or(ApiError::EnvNotFound(env_id.clone()))?;

    sqlx::query!(
        "UPDATE envs SET next_deploy_seq = next_deploy_seq + 1 WHERE id = ?",
        env_id
    )
    .execute(&mut txn)
    .await
    .context("Failed to update envs table")?;

    let deploy_seq = env.next_deploy_seq + 1;
    let deploy_id = new_nano_id();
    sqlx::query!(
        "INSERT INTO deploys (id, updated_at, tag, description, env_id, deploy_seq) VALUES (?, CURRENT_TIMESTAMP(3), ?, ?, ?, ?)",
        deploy_id,
        req.tag,
        req.desc,
        env_id,
        deploy_seq,
    )
        .execute(&mut txn)
        .await
        .context("Failed to insert into deploys table")?;

    let mut codes = vec![];
    for code in req.codes.iter() {
        let code_id = new_nano_id();
        sqlx::query!(
            "INSERT INTO codes (id, updated_at, deploy_id, fs_path, content, content_size) VALUES (?, CURRENT_TIMESTAMP(3), ?, ?, ?, ?)",
            code_id,
            deploy_id,
            code.fs_path,
            code.content,
            code.content.len() as i64,
        )
            .execute(&mut txn)
            .await
            .context("Failed to insert into bundles table")?;
        codes.push(Code {
            fs_path: code.fs_path.clone(),
            content: code.content.clone(),
        });

        tracing::debug!(
            env = env_id,
            seq = deploy_seq,
            "add bundle {}",
            code.fs_path.as_str(),
        );
    }

    let registry_code_content = registry_code(&http_routes)?;
    let registry_code_id = new_nano_id();
    sqlx::query!(
        "INSERT INTO codes (id, updated_at, deploy_id, fs_path, content, content_size) VALUES (?, CURRENT_TIMESTAMP(3), ?, ?, ?, ?)",
        registry_code_id,
        deploy_id,
        REGISTRY_FILE_NAME,
        registry_code_content,
        registry_code_content.len() as i64,
    ).execute(&mut txn).await.context("Failed to insert registry code")?;

    codes.push(Code {
        fs_path: REGISTRY_FILE_NAME.to_string(),
        content: registry_code_content,
    });
    // create new http_routes
    for route in http_routes.iter() {
        let route_id = new_nano_id();
        sqlx::query!(
            "INSERT INTO http_routes (id, updated_at, method, js_entry_point, js_export, deploy_id, http_path, func_sig_version, func_sig) VALUES (?, CURRENT_TIMESTAMP(3), ?, ?, ?, ?, ?, ?, ?)",
            route_id,
            "POST",
            route.js_entry_point,
            route.js_export,
            deploy_id,
            route.http_path,
            route.func_sig_version,
            serde_json::to_string(&route.func_sig).context("Failed to serialize func_sig")?,
        ).execute(&mut txn).await.context("Failed to insert into http_routes table")?;

        tracing::debug!(
            env = env_id,
            seq = deploy_seq,
            "add route {}",
            &route.http_path
        );
    }
    txn.commit()
        .await
        .context("Failed to commit database transaction")?;

    tracing::info!(
        env = env_id,
        seq = deploy_seq,
        "added deployment, {} bundles, {} routes",
        codes.len(),
        http_routes.len()
    );

    let req = AddDeploymentReq {
        env_id,
        deploy_seq,
        codes,
        http_routes: http_routes.clone(),
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
    Ok(Json(DeployCodeRsp { http_routes }))
}

async fn list_code(
    server_state: Data<Arc<ServerState>>,
    env_id: Path<String>,
) -> Result<Json<ListCodeRsp>, ApiError> {
    let db_pool = &server_state.db_pool;
    let env_id = env_id.into_inner();
    let deploy_id = sqlx::query!(
        "SELECT id FROM deploys WHERE env_id = ? ORDER BY deploy_seq DESC LIMIT 1",
        env_id
    ).fetch_optional(db_pool).await.context("Failed to query deploys table")?;
    let mut codes = vec![];
    let mut http_routes = vec![];
    if let Some(deploy_id) = deploy_id {
        let records = sqlx::query!("
        SELECT id, fs_path, content FROM codes WHERE deploy_id = ? AND fs_path != '__registry.js'",
            deploy_id.id).fetch_all(db_pool).await.context("Failed to query bundles table")?;
        for r in records.iter() {
            codes.push(Code {
                fs_path: r.fs_path.clone(),
                content: String::from_utf8(r.content.clone()).map_err(|e| {
                    ApiError::Internal(anyhow!(
                        "Failed to convert code content to string: {}",
                        e
                    ))
                })?,
            });
        }
        let routes = sqlx::query!("\
        SELECT http_path, method, js_entry_point, js_export, func_sig_version, func_sig FROM http_routes WHERE deploy_id = ?",
            deploy_id.id).fetch_all(db_pool).await.context("Failed to query http_routes table")?;
        for r in routes.iter() {
            http_routes.push(HttpRoute {
                http_path: r.http_path.clone(),
                method: r.method.clone(),
                js_entry_point: r.js_entry_point.clone(),
                js_export: r.js_export.clone(),
                func_sig_version: r.func_sig_version,
                func_sig: serde_json::from_value(r.func_sig.clone()).map_err(
                    |e| {
                        ApiError::Internal(anyhow!(
                            "Failed to parse func_sig: {}",
                            e
                        ))
                    },
                )?,
            });
        }
    }
    Ok(Json(ListCodeRsp { codes, http_routes }))
}

const REGISTRY_TEMPLATE: &str = r#"
{{#each routes}}
import { {{js_export}} as {{ unique_export }} } from "./{{js_entry_point}}";
globalThis.{{unique_export}} = {{unique_export}};
{{/each}}
"#;

fn registry_code(routes: &Vec<HttpRoute>) -> Result<String> {
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

    tracing::debug!("registry {}", &code);

    Ok(code)
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
    use darx_core::FunctionSignatureV1;

    #[test]
    fn test_registry_code() -> Result<()> {
        let routes = vec![
            HttpRoute {
                http_path: "foo".to_string(),
                method: "POST".to_string(),
                js_entry_point: "foo.js".to_string(),
                js_export: "default".to_string(),
                func_sig_version: 1,
                func_sig: FunctionSignatureV1 {
                    export_name: "default".to_string(),
                    param_names: vec![],
                },
            },
            HttpRoute {
                http_path: "foo.foo".to_string(),
                method: "POST".to_string(),
                js_entry_point: "foo.js".to_string(),
                js_export: "foo".to_string(),
                func_sig_version: 1,
                func_sig: FunctionSignatureV1 {
                    export_name: "foo".to_string(),
                    param_names: vec![],
                },
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
            code
        );
        Ok(())
    }
}
