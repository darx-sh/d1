use std::cell::RefCell;
use std::env;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::OnceLock;
use std::time::Duration;

use actix_cors::Cors;
use actix_web::dev::{ConnectionInfo, Server};
use actix_web::web::{get, post, Json, Path};
use actix_web::{
    App, HttpRequest, HttpResponse, HttpResponseBuilder, HttpServer,
};
use anyhow::{Context, Result};
use deno_core::{serde_v8, v8};
use handlebars::Handlebars;
use serde_json;
use serde_json::json;
use tokio::fs;
use tracing::info;
use tracing_actix_web::TracingLogger;

use darx_api::{unique_js_export, AddDeploymentReq, ApiError};
use darx_isolate_runtime::DarxIsolate;

use crate::cache::LruCache;
use crate::deployment::{
    add_bundle_files, add_route, find_bundle_dir, init_deployments,
    match_route, DeploymentRoute, SNAPSHOT_FILE,
};

mod cache;
mod deployment;

const DARX_BUNDLES_DIR: &str = "./darx_bundles";

static BUNDLES_DIR: OnceLock<PathBuf> = OnceLock::new();

//TODO lru size should be configured
type IsolateCache = LruCache<PathBuf, DarxIsolate, 100>;

thread_local! {
    static CACHE : Rc<RefCell<IsolateCache>> = Rc::new(RefCell::new(IsolateCache::new()));
}

pub async fn run_server(
    socket_addr: SocketAddr,
    working_dir: PathBuf,
) -> Result<Server> {
    fs::create_dir_all(working_dir.as_path())
        .await
        .context("Failed to create working directory")?;
    let working_dir = fs::canonicalize(working_dir)
        .await
        .context("Failed to canonicalize working dir")?;
    let bundles_dir =
        BUNDLES_DIR.get_or_init(|| working_dir.join(DARX_BUNDLES_DIR));
    fs::create_dir_all(bundles_dir.as_path()).await?;

    let db_pool = sqlx::MySqlPool::connect(
        env::var("DATABASE_URL")
            .expect("DATABASE_URL should be configured")
            .as_str(),
    )
    .await
    .context("Failed to connect database")?;

    init_deployments(bundles_dir.as_path(), &db_pool)
        .await
        .context("Failed to init deployments on startup")?;

    info!("listen on {}", socket_addr);

    Ok(HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_method()
            .allow_any_header()
            .allow_any_origin();
        App::new()
            .wrap(TracingLogger::default())
            .wrap(cors)
            .route("/", get().to(|| async { "data plane healthy." }))
            .route("/invoke/{func_url}", post().to(invoke_function))
            .route("/add_deployment", post().to(add_deployment))
    })
    .bind(&socket_addr)?
    .run())
}

async fn invoke_function0(
    env_id: &str,
    deploy_seq: i32,
    req: serde_json::Value,
    js_entry_point: &str,
    js_export: &str,
    param_names: &Vec<String>,
) -> Result<serde_json::Value, ApiError> {
    let bundle_dir =
        find_bundle_dir(BUNDLES_DIR.get().unwrap(), env_id, deploy_seq)
            .await
            .map_err(|e| ApiError::BundleNotFound(e.to_string()))?;

    let snapshot_path = bundle_dir.join(SNAPSHOT_FILE);
    let cache = CACHE.with(Rc::clone);
    let mut cache = cache.borrow_mut();
    let cached = cache.get_mut(&snapshot_path);
    // let isolate = if cached.is_none() {
    //     debug!("cache miss, cur size {}", cache.len());
    //
    //     let snapshot =
    //         fs::read(&snapshot_path).await.map_err(ApiError::IoError)?;
    //
    //     let isolate = DarxIsolate::new_with_snapshot(
    //         env_id,
    //         deploy_seq,
    //         bundle_dir,
    //         snapshot.into_boxed_slice(),
    //     )
    //     .await;
    //     cache.put(snapshot_path.clone(), isolate);
    //     cache.get_mut(&snapshot_path).unwrap()
    // } else {
    //     cached.unwrap()
    // };

    let snapshot = fs::read(&snapshot_path).await.map_err(ApiError::IoError)?;

    let mut isolate = DarxIsolate::new_with_snapshot(
        env_id,
        deploy_seq,
        bundle_dir,
        snapshot.into_boxed_slice(),
    )
    .await;

    let script_result = isolate
        .js_runtime
        .execute_script(
            "invoke_function",
            invoking_code(
                unique_js_export(js_entry_point, js_export),
                param_names.clone(),
                req,
            )?,
        )
        .map_err(ApiError::Internal)?;

    let script_result = isolate.js_runtime.resolve_value(script_result);

    //TODO timeout from env vars/config
    let duration = Duration::from_secs(5);

    let script_result =
        match tokio::time::timeout(duration, script_result).await {
            Err(_) => Err(ApiError::Timeout),
            Ok(res) => res.map_err(ApiError::Internal),
        }?;

    let mut handle_scope = isolate.js_runtime.handle_scope();
    let script_result = v8::Local::new(&mut handle_scope, script_result);
    let script_result = serde_v8::from_v8(&mut handle_scope, script_result)
        .map_err(|e| ApiError::Internal(anyhow::Error::new(e)))?;
    Ok(script_result)
}

async fn invoke_function(
    conn: ConnectionInfo,
    func_url: Path<String>,
    http_req: HttpRequest,
    Json(req): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let darx_env = env::var("DARX_ENV").expect("DARX_ENV should be configured");
    let host = if darx_env != "production" {
        // dev environment use a special Darx-Dev-Host header to specify the host.
        http_req
            .headers()
            .get("Darx-Dev-Host")
            .map_or("".to_string(), |v| {
                v.to_str()
                    .context(
                        "Darx-Dev-Host contains non visible ascii characters",
                    )
                    .unwrap()
                    .to_string()
            })
    } else {
        conn.host().to_string()
    };
    let env_id = extract_env_id(host.as_str())?;
    let func_url = func_url.into_inner();

    let (deploy_seq, route) =
        match_route(env_id.as_str(), func_url.as_str(), "POST").ok_or(
            ApiError::FunctionNotFound(format!(
                "host: {}, env_id: {}",
                host, &env_id
            )),
        )?;

    let ret = invoke_function0(
        &env_id,
        deploy_seq,
        req,
        &route.js_entry_point,
        &route.js_export,
        &route.func_sig.param_names,
    )
    .await?;
    Ok(Json(ret))
}

async fn add_deployment(
    Json(req): Json<AddDeploymentReq>,
) -> Result<HttpResponseBuilder, ApiError> {
    let bundles_len = req.codes.len();
    let routes_len = req.http_routes.len();

    let mut deployment_route = DeploymentRoute {
        env_id: req.env_id.clone(),
        deploy_seq: req.deploy_seq,
        http_routes: Default::default(),
    };

    for r in req.http_routes {
        deployment_route.http_routes.insert(r.http_path.clone(), r);
    }

    add_bundle_files(
        req.env_id.as_str(),
        req.deploy_seq,
        BUNDLES_DIR.get().unwrap(),
        &req.codes,
    )
    .await?;
    add_route(deployment_route);

    info!(
        env = req.env_id.as_str(),
        seq = req.deploy_seq,
        "cached deployment, {} bundles, {} routes",
        bundles_len,
        routes_len
    );
    Ok(HttpResponse::Ok())
}

const INVOKING_TEMPLATE: &str = r#"
// an object of parameters
const paramValuesJson = '{{{param_values_json}}}';

// an array of function parameter names
const paramNamesJson = '{{{param_names_json}}}'; 

const paramValues = JSON.parse(paramValuesJson);
const paramNames = JSON.parse(paramNamesJson);

const paramsArray = paramNames.map((name) => {
    return paramValues[name];
});

{{func_name}}(...paramsArray);
"#;

fn invoking_code(
    func_name: String,
    param_names: Vec<String>,
    param_values: serde_json::Value,
) -> Result<String> {
    let reg = Handlebars::new();
    let code = reg.render_template(
        INVOKING_TEMPLATE,
        &json!({
            "func_name": func_name,
            "param_values_json": serde_json::to_string(&param_values).unwrap(),
            "param_names_json": serde_json::to_string(&param_names).unwrap(),
        }),
    )?;
    Ok(code)
}

fn extract_env_id(domain: &str) -> Result<String> {
    let mut parts = domain.split('.');
    let env_id = parts.next().ok_or_else(|| {
        ApiError::DomainNotFound(format!("invalid domain: {}", domain))
    })?;
    Ok(env_id.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invoking_code_simple() -> Result<()> {
        let code = invoking_code(
            "foo".to_string(),
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
            json!({
                "a": [1, 2, 3],
                "b": "hello",
                "c": 3,
            }),
        )?;
        println!("{}", code);
        Ok(())
    }
}
