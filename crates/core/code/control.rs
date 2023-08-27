use crate::api::{ApiError, EnvInfo, ProjectInfo};
use crate::code::esm_parser::parse_module_export;
use crate::env_vars::var::{Var, VarKind};
use crate::env_vars::var_list::VarList;
use crate::plugin::plugin_http_path;
use crate::route_builder::build_route;
use crate::{
  unique_js_export, Code, DeployId, DeploySeq, HttpRoute, REGISTRY_FILE_NAME,
};
use anyhow::{anyhow, Context, Result};
use darx_utils::new_nano_id;
use handlebars::Handlebars;
use serde::Serialize;
use serde_json::json;
use sqlx::{MySql, MySqlPool, Transaction};
use std::collections::{BTreeMap, HashMap};
use std::mem::swap;

pub async fn deploy_code<'c>(
  txn: Transaction<'c, MySql>,
  env_id: &str,
  codes: &Vec<Code>,
  tag: &Option<String>,
  desc: &Option<String>,
) -> Result<(i64, Vec<Code>, Vec<HttpRoute>, Transaction<'c, MySql>), ApiError>
{
  let mut http_routes = vec![];
  for code in codes.iter() {
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
    let fn_sigs = parse_module_export(code.fs_path.as_str(), content.as_str())
      .map_err(|e| ApiError::FunctionParseError(e.to_string()))?;
    for sig in fn_sigs.iter() {
      // the http route should start without `functions`.
      let route = build_route(Some(functions_dir), code.fs_path.as_str(), sig)?;
      http_routes.push(route);
    }
  }

  let (deploy_id, deploy_seq, mut txn) =
    create_deploy(txn, env_id, tag, desc).await?;

  let mut final_codes = vec![];
  for code in codes.iter() {
    let code_id = new_nano_id();
    sqlx::query!(
            "INSERT INTO codes (id, updated_at, deploy_id, fs_path, content, content_size) VALUES (?, CURRENT_TIMESTAMP(3), ?, ?, ?, ?)",
            code_id,
            deploy_id,
            code.fs_path,
            code.content,
            code.content.len() as i64,
        )
            .execute(&mut *txn)
            .await
            .context("Failed to insert into codes table")?;
    final_codes.push(Code {
      fs_path: code.fs_path.clone(),
      content: code.content.clone(),
    });

    tracing::debug!(
      env = env_id,
      seq = deploy_seq,
      "add code {}",
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
    ).execute(&mut *txn).await.context("Failed to insert registry code")?;

  final_codes.push(Code {
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
        ).execute(&mut *txn).await.context("Failed to insert into http_routes table")?;

    tracing::debug!(
      env = env_id,
      seq = deploy_seq,
      "add route {}",
      &route.http_path
    );
  }

  tracing::info!(
    env = env_id,
    seq = deploy_seq,
    "added deployment, {} codes, {} routes",
    final_codes.len(),
    http_routes.len()
  );
  Ok((deploy_seq, final_codes, http_routes, txn))
}

pub async fn deploy_var<'c>(
  txn: Transaction<'c, MySql>,
  env_id: &str,
  vars: &HashMap<String, String>,
  desc: &Option<String>,
) -> Result<(DeploySeq, HashMap<String, String>, Transaction<'c, MySql>)> {
  let (deploy_id, deploy_seq, mut txn) =
    create_deploy(txn, env_id, &None, desc).await?;

  let var_list = VarList::find(&mut *txn, env_id, VarKind::Env)
    .await
    .context("Failed to find env vars")?;
  let mut var_list = var_list.env_to_deploy(&deploy_id);
  let mut map: BTreeMap<&str, &str> =
    var_list.vars().iter().map(|e| (e.key(), e.val())).collect();

  // merge deploy_vars with env_vars
  for var in vars {
    map.insert(var.0, var.1);
  }
  let mut final_vars: Vec<Var> =
    map.into_iter().map(|(k, v)| Var::new(k, v)).collect();
  swap(var_list.mut_vars(), &mut final_vars);

  var_list
    .save(&mut *txn)
    .await
    .context("Fail to save deploy vars")?;
  Ok((deploy_seq, var_list.into_map(), txn))
}

pub async fn load_env(
  db_pool: &MySqlPool,
  proj_id: &str,
) -> Result<(Vec<Code>, Vec<HttpRoute>, ProjectInfo, EnvInfo)> {
  let project =
    sqlx::query!("SELECT id, name FROM projects WHERE id = ?", proj_id)
      .fetch_optional(db_pool)
      .await
      .context("Failed to query projects table")?
      .ok_or(ApiError::ProjectNotFound(proj_id.to_string()))?;

  // todo: only fetch the first environment right now.
  let env =
    sqlx::query!("SELECT id, name FROM envs WHERE project_id = ?", project.id)
      .fetch_optional(db_pool)
      .await
      .context("Failed to query envs table")?
      .ok_or(ApiError::EnvNotFound(format!("project_id: {}", proj_id)))?;

  let env_id = env.id.clone();
  let proj_info = ProjectInfo {
    id: project.id,
    name: project.name,
  };
  let env_info = EnvInfo {
    id: env_id.clone(),
    name: env.name,
  };

  let deploy_id = sqlx::query!(
    "SELECT id FROM deploys WHERE env_id = ? ORDER BY deploy_seq DESC LIMIT 1",
    env_id
  )
  .fetch_optional(db_pool)
  .await
  .context("Failed to query deploys table")?;
  let mut codes = vec![];
  let mut http_routes = vec![];
  if let Some(deploy_id) = deploy_id {
    let records = sqlx::query!("
        SELECT id, fs_path, content FROM codes WHERE deploy_id = ? AND fs_path != '__registry.js'",
            deploy_id.id).fetch_all(db_pool).await.context("Failed to query codes table")?;
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
        func_sig: serde_json::from_value(r.func_sig.clone()).map_err(|e| {
          ApiError::Internal(anyhow!("Failed to parse func_sig: {}", e))
        })?,
      });
    }
  }
  Ok((codes, http_routes, proj_info, env_info))
}

pub async fn deploy_plugin<'c>(
  mut txn: Transaction<'c, MySql>,
  env_id: &str,
  plugin_name: &str,
  codes: &Vec<Code>,
) -> Result<(DeploySeq, Vec<Code>, Vec<HttpRoute>, Transaction<'c, MySql>), ApiError> {
  let plugin = sqlx::query!("SELECT id FROM plugins WHERE name = ?", plugin_name)
    .fetch_optional(&mut *txn)
    .await
    .context("Failed to query plugins table")?;

  let _plugin_id = if let Some(plugin) = plugin {
    plugin.id
  } else {
    let plugin_id = new_nano_id();
    sqlx::query!(
      "INSERT INTO plugins (id, name, env_id) VALUES (?, ?, ?)",
      plugin_id,
      plugin_name,
      env_id,
    )
    .execute(&mut *txn)
    .await
    .context("Failed to insert into plugins table")?;
    plugin_id
  };
  deploy_code(txn, env_id, codes, &None, &None).await
}

pub async fn list_api(
  db_pool: &MySqlPool,
  env_id: &str,
) -> Result<Vec<HttpRoute>> {
  let mut http_routes = api_for_env(db_pool, env_id).await?;

  let plugins = sqlx::query!("SELECT name, env_id FROM plugins")
    .fetch_all(db_pool)
    .await
    .context("Failed to query plugins table")?;
  for p in plugins.iter() {
    let routes = api_for_env(db_pool, &p.env_id).await?;
    for r in routes {
      http_routes.push(HttpRoute {
        http_path: plugin_http_path(&p.name, &r.http_path),
        ..r
      });
    }
  }
  Ok(http_routes)
}

async fn create_deploy<'c>(
  mut txn: Transaction<'c, MySql>,
  env_id: &str,
  tag: &Option<String>,
  desc: &Option<String>,
) -> Result<(DeployId, DeploySeq, Transaction<'c, MySql>)> {
  // create new deploy
  let env = sqlx::query!(
    "SELECT next_deploy_seq FROM envs WHERE id = ? FOR UPDATE",
    env_id
  )
  .fetch_optional(&mut *txn)
  .await
  .context("Failed to find env")?
  .ok_or(ApiError::EnvNotFound(env_id.to_string()))?;
  let deploy_seq = env.next_deploy_seq;

  sqlx::query!(
    "UPDATE envs SET next_deploy_seq = next_deploy_seq + 1 WHERE id = ?",
    env_id
  )
  .execute(&mut *txn)
  .await
  .context("Failed to update envs table")?;

  let deploy_id = new_nano_id();
  sqlx::query!(
        "INSERT INTO deploys (id, updated_at, tag, description, env_id, deploy_seq) VALUES (?, CURRENT_TIMESTAMP(3), ?, ?, ?, ?)",
        deploy_id,
        tag,
        desc,
        env_id,
        deploy_seq,
    )
    .execute(&mut *txn)
    .await
    .context("Failed to insert into deploys table")?;
  Ok((deploy_id, deploy_seq, txn))
}

async fn api_for_env(
  db_pool: &MySqlPool,
  env_id: &str,
) -> Result<Vec<HttpRoute>> {
  let mut http_routes = vec![];
  let deploy_id = sqlx::query!(
    "SELECT id FROM deploys WHERE env_id = ? ORDER BY deploy_seq DESC LIMIT 1",
    env_id
  )
  .fetch_optional(db_pool)
  .await
  .context("Failed to query deploys table")?;
  if let Some(deploy_id) = deploy_id {
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
        func_sig: serde_json::from_value(r.func_sig.clone()).map_err(|e| {
          ApiError::Internal(anyhow!("Failed to parse func_sig: {}", e))
        })?,
      });
    }
  }
  Ok(http_routes)
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
  let code = reg
    .render_template(REGISTRY_TEMPLATE, &json!({ "routes": unique_imports }))?;

  tracing::debug!("registry {}", &code);

  Ok(code)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::FunctionSignatureV1;

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
