use anyhow::{bail, Context, Result};
use darx_db::{add_tenant_db_info, TenantDBInfo};
use dashmap::DashMap;
use deno_core::{serde_v8, v8};
use futures::TryStreamExt;
use once_cell::sync::Lazy;
use patricia_tree::StringPatriciaMap;
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::Duration;
use tokio::fs;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::time::Instant;
use tracing::{debug, info};

use darx_isolate_runtime::{build_snapshot, DarxIsolate};

use crate::api::ApiError;
use crate::tenants::cache::LruCache;
use crate::{
  plugin, unique_js_export, Code, DeploySeq, HttpRoute, REGISTRY_FILE_NAME,
};

//TODO lru size should be configured
type SnapshotCache = LruCache<PathBuf, Box<[u8]>, 100>;

thread_local! {
    static CACHE : Rc<RefCell<SnapshotCache >> = Rc::new(RefCell::new(SnapshotCache::new()));
}

#[derive(Clone, Debug)]
pub(crate) struct RouteDeploy {
  pub env_id: String,
  pub deploy_seq: DeploySeq,
  pub http_routes: StringPatriciaMap<HttpRoute>,
}

pub(crate) struct VarDeploy {
  pub env_id: String,
  pub deploy_seq: DeploySeq,
  pub vars: HashMap<String, String>,
}

static GLOBAL_ROUTER: Lazy<DashMap<String, Vec<RouteDeploy>>> =
  Lazy::new(DashMap::new);

static GLOBAL_VARS: Lazy<DashMap<String, Vec<VarDeploy>>> =
  Lazy::new(DashMap::new);

// PLUGIN_REGISTRY maps a plugin's name to its env_id.
static PLUGIN_REGISTRY: Lazy<DashMap<String, String>> = Lazy::new(DashMap::new);

pub(crate) const SNAPSHOT_FILE: &str = "SNAPSHOT.bin";

pub async fn add_plugin_deploy(
  name: &str,
  envs_dir: &Path,
  env_id: &str,
  deploy_seq: i64,
  codes: &Vec<Code>,
  http_routes: &Vec<HttpRoute>,
) -> Result<()> {
  let kv = PLUGIN_REGISTRY
    .entry(name.to_string())
    .or_insert(env_id.to_string());
  if kv.value() != env_id {
    bail!("Plugin name {} already exists in env {}", name, kv.value());
  }
  add_code_deploy(envs_dir, env_id, deploy_seq, codes, http_routes).await?;
  Ok(())
}

pub async fn add_code_deploy(
  envs_dir: &Path,
  env_id: &str,
  deploy_seq: i64,
  codes: &Vec<Code>,
  http_routes: &Vec<HttpRoute>,
) -> Result<()> {
  let code_cnt = codes.len();
  let route_cnt = http_routes.len();

  let mut routes = RouteDeploy {
    env_id: env_id.to_string(),
    deploy_seq,
    http_routes: Default::default(),
  };

  for r in http_routes {
    routes.http_routes.insert(r.http_path.clone(), r.clone());
  }
  add_code_files(env_id, deploy_seq, envs_dir, codes).await?;
  add_route(routes);

  info!(
    env = env_id,
    seq = deploy_seq,
    "cached deployment, {} codes, {} routes",
    code_cnt,
    route_cnt
  );
  Ok(())
}

pub async fn add_var_deploy(
  env_id: &str,
  deploy_seq: DeploySeq,
  vars: &HashMap<String, String>,
) -> Result<()> {
  let vars = VarDeploy {
    env_id: env_id.to_string(),
    deploy_seq,
    vars: vars.clone(),
  };

  let mut entry = GLOBAL_VARS
    .entry(env_id.to_string())
    .or_insert_with(|| Vec::new());
  // newest deploy_seq stores in the front of the array
  entry.insert(0, vars);
  entry.sort_by(|a, b| b.deploy_seq.cmp(&a.deploy_seq));
  Ok(())
}

pub async fn init_deploys(
  envs_dir: &Path,
  pool: &sqlx::MySqlPool,
) -> Result<()> {
  let mut plugins =
    sqlx::query!("SELECT env_id, name FROM plugins").fetch(pool);
  while let Some(plugin) = plugins.try_next().await? {
    PLUGIN_REGISTRY.insert(plugin.name.clone(), plugin.env_id.clone());
  }

  // setup GLOBAL_ROUTER
  let mut deploys = sqlx::query!(
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
  .fetch(pool);
  while let Some(deploy) = deploys.try_next().await? {
    let http_route = HttpRoute {
      http_path: deploy.http_path.clone(),
      js_entry_point: deploy.js_entry_point.clone(),
      js_export: deploy.js_export.clone(),
      method: deploy.method.clone(),
      func_sig_version: deploy.func_sig_version,
      func_sig: serde_json::from_value(deploy.func_sig.clone())
        .context("Failed to extract func_sig")?,
    };
    add_one_http_route(deploy.env_id.as_str(), deploy.deploy_seq, http_route);
  }

  // setup source code and snapshot in file system.
  let mut codes = sqlx::query!(
    "SELECT \
            deploys.env_id AS env_id, \
            deploys.deploy_seq AS deploy_seq, \
            codes.id AS id, \
            codes.fs_path AS fs_path, \
            codes.content AS content \
        FROM \
            codes INNER JOIN deploys ON deploys.id = codes.deploy_id"
  )
  .fetch(pool);
  let mut map: HashMap<(String, DeploySeq), Vec<Code>> = HashMap::new();
  while let Some(code) = codes.try_next().await? {
    map
      .entry((code.env_id.to_string(), code.deploy_seq))
      .or_default()
      .push(Code {
        fs_path: code.fs_path.clone(),
        content: String::from_utf8(code.content.clone())?,
      });
  }

  for ((env_id, deploy_seq), v) in &map {
    for b in v {
      add_single_source_code(envs_dir, env_id, *deploy_seq, b)
        .await
        .with_context(|| {
          format!(
            "Failed to add code file on startup. env_id: {}, deploy_seq: {}",
            env_id, deploy_seq,
          )
        })?;
    }

    add_snapshot(envs_dir, env_id, *deploy_seq).await?;
  }

  // setup GLOBAL_VARS
  let mut vars = sqlx::query!(
    "SELECT \
            deploys.env_id AS env_id, \
            deploys.deploy_seq AS deploy_seq, \
            deploy_vars.key AS `key`, \
            deploy_vars.value AS value \
        FROM \
            deploy_vars INNER JOIN deploys ON deploys.id = deploy_vars.deploy_id"
  ).fetch(pool);
  while let Some(row) = vars.try_next().await? {
    add_one_var(
      row.env_id.as_str(),
      row.deploy_seq,
      row.key.as_str(),
      row.value.as_str(),
    );
  }

  // setup GLOBAL_DB_INFO
  let mut dbs = sqlx::query!(
    "SELECT env_id, db_host, db_port, db_user, db_password, db_name FROM env_dbs"
  )
  .fetch(pool);
  while let Some(db) = dbs.try_next().await? {
    let db_info = TenantDBInfo {
      host: db.db_host,
      port: db.db_port as u16,
      user: db.db_user,
      password: db.db_password,
      database: db.db_name,
    };
    add_tenant_db_info(db.env_id.as_str(), db_info);
  }

  Ok(())
}

/// [`match_route`] returns (env_id, deploy_seq, http_route).
/// The returned env_id might not be the same as the input env_id,
/// this only happens when the [`func_url`] starts with [`_plugins`] which
/// tells the router to load the plugin from the plugin's env directory.
pub fn match_route(
  env_id: &str,
  func_url: &str,
  method: &str,
) -> Option<(String, i64, HttpRoute)> {
  let (env_id, func_url) = if func_url.starts_with("_plugins/") {
    let res = plugin::parse_plugin_url(func_url);
    if res.is_err() {
      return None;
    }
    let (plugin_name, func_url) = res.unwrap();
    if let Some(env_id) = lookup_plugin(plugin_name.as_str()) {
      (env_id, func_url)
    } else {
      return None;
    }
  } else {
    (env_id.to_string(), func_url.to_string())
  };

  if let Some(entry) = GLOBAL_ROUTER.get(env_id.as_str()) {
    //TODO multi-version support
    let cur_deploy = &entry[0];

    if let Some(r) = cur_deploy.http_routes.get(func_url.as_str()) {
      debug_assert!(r.http_path == func_url.as_str());
      debug_assert!(r.method == method);
      Some((env_id.to_string(), cur_deploy.deploy_seq, r.clone()))
    } else {
      None
    }
  } else {
    None
  }
}

///
/// [`find_vars`] returns the vars for the given env_id and code's deploy_seq.
/// The returned vars has the highest deploy_seq.
///
pub fn find_vars(env_id: &str) -> Option<HashMap<String, String>> {
  if let Some(entry) = GLOBAL_VARS.get(env_id) {
    if let Some(v) = entry.get(0) {
      return Some(v.vars.clone());
    }
    return None;
  }
  None
}

pub async fn invoke_function(
  envs_dir: &Path,
  env_id: &str,
  target_env_id: &str,
  deploy_seq: i64,
  req: serde_json::Value,
  js_entry_point: &str,
  js_export: &str,
  param_names: &Vec<String>,
) -> Result<serde_json::Value, ApiError> {
  let deploy_dir = find_deploy_dir(envs_dir, target_env_id, deploy_seq)
    .await
    .map_err(|e| ApiError::DeployNotFound(e.to_string()))?;

  let snapshot_path = deploy_dir.join(SNAPSHOT_FILE);
  let cache = CACHE.with(Rc::clone);
  let mut cache = cache.borrow_mut();
  let cached = cache.get_mut(&snapshot_path);
  let snapshot = if cached.is_none() {
    debug!("cache miss, cur size {}", cache.len());

    let snapshot = fs::read(&snapshot_path).await.map_err(ApiError::IoError)?;

    cache.put(snapshot_path.clone(), snapshot.into_boxed_slice());
    cache.get(&snapshot_path).unwrap().clone()
  } else {
    cached.unwrap().clone()
  };

  // We DO NOT use target_env here.
  // We use the env_id from the request.
  let vars = find_vars(env_id).unwrap_or_default();
  let mut isolate = DarxIsolate::new_with_snapshot(
    env_id,
    deploy_seq,
    &vars,
    &deploy_dir,
    snapshot,
  )
  .await;

  let source_code = invoking_code(
    unique_js_export(js_entry_point, js_export),
    param_names.clone(),
    req,
  )?;
  let script_result = isolate
    .js_runtime
    .execute_script("invoke_function", source_code)
    .context("execute script error")
    .map_err(ApiError::FunctionRuntimeError)?;

  let script_result = isolate.js_runtime.resolve_value(script_result);

  //TODO timeout from env vars/config
  let duration = Duration::from_secs(5);

  let script_result = match tokio::time::timeout(duration, script_result).await
  {
    Err(_) => Err(ApiError::Timeout),
    Ok(res) => res
      .context("resolve value error")
      .map_err(ApiError::FunctionRuntimeError),
  }?;

  let mut handle_scope = isolate.js_runtime.handle_scope();
  let script_result = v8::Local::new(&mut handle_scope, script_result);
  let script_result = serde_v8::from_v8(&mut handle_scope, script_result)
    .context("deserialize result error")
    .map_err(ApiError::Internal)?;
  Ok(script_result)
}

fn add_route(route: RouteDeploy) {
  let env_id = route.env_id.clone();
  let mut entry = GLOBAL_ROUTER.entry(env_id).or_insert_with(|| Vec::new());
  // newest deploy_seq stores in the front of the array
  entry.insert(0, route.clone());
  entry.sort_by(|a, b| b.deploy_seq.cmp(&a.deploy_seq));
}

async fn add_code_files(
  env_id: &str,
  deploy_seq: i64,
  code_root_dir: impl AsRef<Path>,
  codes: &Vec<Code>,
) -> Result<()> {
  // setup bundle files
  for code in codes.iter() {
    add_single_source_code(code_root_dir.as_ref(), env_id, deploy_seq, code)
      .await?;
  }
  add_snapshot(code_root_dir.as_ref(), env_id, deploy_seq).await?;
  Ok(())
}

async fn add_single_source_code(
  envs_dir: &Path,
  env_id: &str,
  deploy_seq: i64,
  code: &Code,
) -> Result<()> {
  let deploy_dir =
    setup_deploy_dir(envs_dir.as_ref(), env_id, deploy_seq).await?;
  let code_file = deploy_dir.join(code.fs_path.as_str());

  if let Some(parent) = code_file.parent() {
    if !parent.exists() {
      fs::create_dir_all(parent).await?;
    }
  }

  let mut file = File::create(code_file.as_path()).await?;
  file.write_all(code.content.as_ref()).await?;
  Ok(())
}

async fn add_snapshot(
  envs_dir: &Path,
  env_id: &str,
  deploy_seq: i64,
) -> Result<()> {
  let deploy_dir =
    setup_deploy_dir(envs_dir.as_ref(), env_id, deploy_seq).await?;
  let mut mark = Instant::now();
  let snapshot =
    build_snapshot(deploy_dir.as_path(), REGISTRY_FILE_NAME).await?;
  let snapshot_slice: &[u8] = &snapshot;

  info!(
    env = env_id,
    seq = deploy_seq,
    "Snapshot size: {}, took {:#?}",
    snapshot_slice.len(),
    Instant::now().saturating_duration_since(mark)
  );

  mark = Instant::now();

  let snapshot_path = deploy_dir.join(SNAPSHOT_FILE);

  fs::write(&snapshot_path, snapshot_slice).await.unwrap();

  info!(
    env = env_id,
    seq = deploy_seq,
    "Snapshot written, took: {:#?} ({})",
    Instant::now().saturating_duration_since(mark),
    snapshot_path.display(),
  );

  Ok(())
}

fn add_one_http_route(env_id: &str, deploy_seq: i64, route: HttpRoute) {
  let mut entry = GLOBAL_ROUTER
    .entry(env_id.to_string())
    .or_insert_with(|| Vec::new());

  if let Some(deploy) = entry
    .iter_mut()
    .find(|deploy| deploy.deploy_seq == deploy_seq)
  {
    deploy.http_routes.insert(route.http_path.clone(), route);
  } else {
    let mut d = RouteDeploy {
      env_id: env_id.to_string(),
      deploy_seq,
      http_routes: StringPatriciaMap::new(),
    };
    d.http_routes.insert(route.http_path.clone(), route);
    entry.push(d);
  }

  entry.sort_by(|a, b| b.deploy_seq.cmp(&a.deploy_seq));
}

fn add_one_var(env_id: &str, deploy_seq: DeploySeq, key: &str, value: &str) {
  let mut entry = GLOBAL_VARS
    .entry(env_id.to_string())
    .or_insert_with(|| Vec::new());

  if let Some(deploy) = entry
    .iter_mut()
    .find(|deploy| deploy.deploy_seq == deploy_seq)
  {
    deploy.vars.insert(key.to_string(), value.to_string());
  } else {
    let mut d = VarDeploy {
      env_id: env_id.to_string(),
      deploy_seq,
      vars: HashMap::new(),
    };
    d.vars.insert(key.to_string(), value.to_string());
    entry.push(d);
  }

  entry.sort_by(|a, b| b.deploy_seq.cmp(&a.deploy_seq));
}

async fn setup_deploy_dir(
  envs_dir: &Path,
  env_id: &str,
  deploy_seq: i64,
) -> Result<PathBuf> {
  let env_dir = envs_dir.join(env_id);
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

async fn find_deploy_dir(
  envs_dir: impl AsRef<Path>,
  env_id: &str,
  deploy_seq: i64,
) -> Result<PathBuf> {
  let path = envs_dir
    .as_ref()
    .join(env_id)
    .join(deploy_seq.to_string().as_str())
    .canonicalize()
    .with_context(|| {
      format!(
        "Failed to canonicalize deploy directory. env_id: {}, deploy_seq: {}",
        env_id, deploy_seq
      )
    })?;
  if !path.exists() {
    bail!(
      "Deploy directory does not exist. env_id: {}, deploy_seq: {}",
      env_id,
      deploy_seq
    );
  }
  Ok(path)
}

fn invoking_code(
  func_name: String,
  param_names: Vec<String>,
  param_values: serde_json::Value,
) -> Result<String> {
  let vals: Vec<String> = param_names
    .into_iter()
    .map(|p| {
      param_values
        .get(&p)
        .unwrap_or(&serde_json::Value::Null)
        .to_string()
    })
    .collect();
  Ok(format!("{}({})", func_name, vals.join(", ")))
}

fn lookup_plugin(name: &str) -> Option<String> {
  let env_id = PLUGIN_REGISTRY.get(name);
  env_id.map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
  use serde_json::json;

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
