use crate::{unique_js_export, Code, HttpRoute, REGISTRY_FILE_NAME};
use anyhow::{bail, Context, Result};
use dashmap::DashMap;
use once_cell::sync::Lazy;
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::time::Instant;
use tracing::info;

use crate::api::ApiError;
use crate::deploy::cache::LruCache;
use darx_isolate_runtime::DarxIsolate;
use deno_core::{serde_v8, v8};
use handlebars::Handlebars;
use patricia_tree::StringPatriciaMap;
use serde_json::json;
use std::rc::Rc;
use std::time::Duration;

//TODO lru size should be configured
type IsolateCache = LruCache<PathBuf, DarxIsolate, 100>;

thread_local! {
    static CACHE : Rc<RefCell<IsolateCache>> = Rc::new(RefCell::new(IsolateCache::new()));
}

#[derive(Clone, Debug)]
pub(crate) struct DeploymentRoute {
    pub env_id: String,
    pub deploy_seq: i32,
    pub http_routes: StringPatriciaMap<HttpRoute>,
}

static GLOBAL_ROUTER: Lazy<DashMap<String, Vec<DeploymentRoute>>> =
    Lazy::new(DashMap::new);

pub(crate) const SNAPSHOT_FILE: &str = "SNAPSHOT.bin";

pub async fn add_deployment(
    bundles_dir: &Path,
    env_id: &str,
    deploy_seq: i32,
    codes: &Vec<Code>,
    http_routes: &Vec<HttpRoute>,
) -> Result<()> {
    let code_cnt = codes.len();
    let route_cnt = http_routes.len();

    let mut deployment_route = DeploymentRoute {
        env_id: env_id.to_string(),
        deploy_seq,
        http_routes: Default::default(),
    };

    for r in http_routes {
        deployment_route
            .http_routes
            .insert(r.http_path.clone(), r.clone());
    }
    add_bundle_files(env_id, deploy_seq, bundles_dir, codes).await?;
    add_route(deployment_route);

    info!(
        env = env_id,
        seq = deploy_seq,
        "cached deployment, {} bundles, {} routes",
        code_cnt,
        route_cnt
    );
    Ok(())
}

pub async fn init_deployments(
    bundles_dir: &Path,
    pool: &sqlx::MySqlPool,
) -> Result<()> {
    let deployments = sqlx::query!(
        "\
    SELECT \
        deploys.id AS deploy_id, \
        deploys.env_id AS env_id, \
        deploys.deploy_seq AS deploy_seq, \
        http_routes.http_path AS http_path, \
        http_routes.js_entry_point AS js_entry_point, \
        http_routes.js_export AS js_export,
        http_routes.method AS method, \
        http_routes.func_sig_version AS func_sig_version, \
        http_routes.func_sig AS func_sig \
    FROM deploys INNER JOIN http_routes ON http_routes.deploy_id = deploys.id"
    )
    .fetch_all(pool)
    .await
    .context("Failed to load bundles from db")?;

    for deploy in deployments.iter() {
        let http_route = HttpRoute {
            http_path: deploy.http_path.clone(),
            js_entry_point: deploy.js_entry_point.clone(),
            js_export: deploy.js_export.clone(),
            method: deploy.method.clone(),
            func_sig_version: deploy.func_sig_version,
            func_sig: serde_json::from_value(deploy.func_sig.clone())
                .context("Failed to extract func_sig")?,
        };

        add_single_http_route(
            deploy.env_id.as_str(),
            deploy.deploy_seq,
            http_route,
        );
    }

    let codes = sqlx::query!(
        "SELECT \
            deploys.env_id AS env_id, \
            deploys.deploy_seq AS deploy_seq, \
            codes.id AS id, \
            codes.fs_path AS fs_path, \
            codes.content AS content \
        FROM \
            codes INNER JOIN deploys ON deploys.id = codes.deploy_id"
    )
    .fetch_all(pool)
    .await
    .context("Failed to load bundles from db")?;

    let mut map: HashMap<(&str, i32), Vec<Code>> =
        HashMap::with_capacity(codes.len());

    for code in codes.iter() {
        map.entry((code.env_id.as_str(), code.deploy_seq))
            .or_default()
            .push(Code {
                fs_path: code.fs_path.clone(),
                content: String::from_utf8(code.content.clone())?,
            });
    }

    for ((env_id, deploy_seq), v) in &map {
        for b in v {
            add_single_source_code(
                bundles_dir,
                env_id,
                *deploy_seq,
                b,
            )
                .await
                .with_context( || {
                    format!(
                        "Failed to add bundle file on startup. env_id: {}, deploy_seq: {}",
                        env_id, deploy_seq,
                    )
                })?;
        }

        add_snapshot(bundles_dir, env_id, *deploy_seq).await?;
    }
    Ok(())
}

pub fn match_route(
    environment_id: &str,
    func_url: &str,
    method: &str,
) -> Option<(i32, HttpRoute)> {
    if let Some(entry) = GLOBAL_ROUTER.get(environment_id) {
        //TODO multi-version support
        let cur_deploy = &entry[0];

        if let Some(r) = cur_deploy.http_routes.get(func_url) {
            debug_assert!(r.http_path == func_url);
            debug_assert!(r.method == method);
            Some((cur_deploy.deploy_seq, r.clone()))
        } else {
            None
        }
    } else {
        None
    }
}

pub async fn invoke_function(
    bundles_dir: &Path,
    env_id: &str,
    deploy_seq: i32,
    req: serde_json::Value,
    js_entry_point: &str,
    js_export: &str,
    param_names: &Vec<String>,
) -> Result<serde_json::Value, ApiError> {
    let bundle_dir = find_bundle_dir(bundles_dir, env_id, deploy_seq)
        .await
        .map_err(|e| ApiError::BundleNotFound(e.to_string()))?;

    let snapshot_path = bundle_dir.join(SNAPSHOT_FILE);
    let cache = CACHE.with(Rc::clone);
    let mut cache = cache.borrow_mut();
    let _cached = cache.get_mut(&snapshot_path);
    // let isolate = if cached.is_none() {
    //     debug!("cache miss, cur size {}", cache.len());
    //
    //     let snapshot =
    //         fs::read(&snapshot_path).await.map_err(ApiError::IoError)?;
    //
    //     let isolate = DarxIsolate::new_with_snapshot(
    //         env_id,
    //         deploy_seq,
    //         bundle_dir.clone(),
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

fn add_route(route: DeploymentRoute) {
    let env_id = route.env_id.clone();
    let mut entry = GLOBAL_ROUTER.entry(env_id).or_insert_with(|| Vec::new());
    entry.insert(0, route.clone());
    entry.sort_by(|a, b| b.deploy_seq.cmp(&a.deploy_seq));
}

async fn add_bundle_files(
    env_id: &str,
    deploy_seq: i32,
    bundles_dir: impl AsRef<Path>,
    bundles: &Vec<Code>,
) -> Result<()> {
    // setup bundle files
    for bundle in bundles.iter() {
        add_single_source_code(
            bundles_dir.as_ref(),
            env_id,
            deploy_seq,
            bundle,
        )
        .await?;
    }
    add_snapshot(bundles_dir.as_ref(), env_id, deploy_seq).await?;
    Ok(())
}

async fn add_single_source_code(
    bundles_dir: &Path,
    env_id: &str,
    deploy_seq: i32,
    code: &Code,
) -> Result<()> {
    let bundle_dir =
        setup_bundle_deployment_dir(bundles_dir.as_ref(), env_id, deploy_seq)
            .await?;
    let bundle_file = bundle_dir.join(code.fs_path.as_str());

    if let Some(parent) = bundle_file.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).await?;
        }
    }

    let mut file = File::create(bundle_file.as_path()).await?;
    file.write_all(code.content.as_ref()).await?;
    Ok(())
}

async fn add_snapshot(
    bundles_dir: &Path,
    env_id: &str,
    deploy_seq: i32,
) -> Result<()> {
    let bundle_dir =
        setup_bundle_deployment_dir(bundles_dir.as_ref(), env_id, deploy_seq)
            .await?;
    let mut js_runtime = DarxIsolate::prepare_snapshot(&bundle_dir).await?;
    let registry_file = bundle_dir.join(REGISTRY_FILE_NAME);
    if !registry_file.exists() {
        tracing::error!(
            env = env_id,
            seq = deploy_seq,
            "file {} does not exist",
            REGISTRY_FILE_NAME,
        );
        bail!("file {} does not exist", REGISTRY_FILE_NAME);
    }

    let module_id = js_runtime
        .load_side_module(
            &deno_core::resolve_path(REGISTRY_FILE_NAME, &bundle_dir)?,
            None,
        )
        .await?;

    let receiver = js_runtime.mod_evaluate(module_id);
    js_runtime.run_event_loop(false).await?;
    let _ = receiver.await?;

    let mut mark = Instant::now();
    let snapshot = js_runtime.snapshot();
    let snapshot_slice: &[u8] = &snapshot;

    tracing::info!(
        env = env_id,
        seq = deploy_seq,
        "Snapshot size: {}, took {:#?}",
        snapshot_slice.len(),
        Instant::now().saturating_duration_since(mark)
    );

    mark = Instant::now();

    let snapshot_path = bundle_dir.join(SNAPSHOT_FILE);

    fs::write(&snapshot_path, snapshot_slice).await.unwrap();

    tracing::info!(
        env = env_id,
        seq = deploy_seq,
        "Snapshot written, took: {:#?} ({})",
        Instant::now().saturating_duration_since(mark),
        snapshot_path.display(),
    );

    Ok(())
}

fn add_single_http_route(env_id: &str, deploy_seq: i32, route: HttpRoute) {
    let mut entry = GLOBAL_ROUTER
        .entry(env_id.to_string())
        .or_insert_with(|| Vec::new());

    if let Some(deploy) = entry
        .iter_mut()
        .find(|deploy| deploy.deploy_seq == deploy_seq)
    {
        deploy.http_routes.insert(route.http_path.clone(), route);
    } else {
        let mut d = DeploymentRoute {
            env_id: env_id.to_string(),
            deploy_seq,
            http_routes: StringPatriciaMap::new(),
        };
        d.http_routes.insert(route.http_path.clone(), route);
        entry.push(d);
    }

    entry.sort_by(|a, b| b.deploy_seq.cmp(&a.deploy_seq));
}

async fn setup_bundle_deployment_dir(
    bundles_dir: &Path,
    env_id: &str,
    deploy_seq: i32,
) -> Result<PathBuf> {
    let env_dir = bundles_dir.join(env_id);
    if !env_dir.exists() {
        fs::create_dir_all(env_dir.as_path())
            .await
            .context("Failed to create env dir")?;
    }

    let deploy_dir = env_dir.join(deploy_seq.to_string().as_str());
    if !deploy_dir.exists() {
        fs::create_dir_all(deploy_dir.as_path())
            .await
            .context("Failed to create deploy dir")?;
    }
    let dir = deploy_dir.canonicalize().with_context(|| {
        format!(
            "Failed to canonicalize deploy directory. env_id: {}, deploy_seq: {}",
            env_id, deploy_seq
        )
    })?;
    Ok(dir)
}

async fn find_bundle_dir(
    bundles_dir: impl AsRef<Path>,
    env_id: &str,
    deploy_seq: i32,
) -> Result<PathBuf> {
    let path = bundles_dir.as_ref().join(env_id).join(deploy_seq.to_string().as_str()).canonicalize().with_context(|| {
        format!(
            "Failed to canonicalize deploy directory. env_id: {}, deploy_seq: {}",
            env_id, deploy_seq
        )
    })?;
    Ok(path)
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
