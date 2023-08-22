mod common;
use anyhow::{Context, Result};
use common::TenantProjectContext;
use darx_core::api::AddCodeDeployReq;
use darx_core::code::control::{deploy_code, deploy_plugin, list_api};
use darx_core::tenants::{
  add_code_deploy, add_plugin_deploy, init_deploys, match_route,
};
use darx_core::{Code, Project};
use darx_utils::new_nano_id;
use sqlx::MySqlPool;
use std::path::PathBuf;
use test_context::test_context;

#[test_context(TenantProjectContext)]
#[tokio::test]
async fn test_deploy_plugin_startup(
  ctx: &mut TenantProjectContext,
) -> Result<()> {
  let envs_dir = PathBuf::from(ctx.envs_dir());
  let db_pool = ctx.db_pool().clone();
  let env_id = ctx.proj().env_id().clone();

  let (plugin_name, plugin_env_id, _, _) = deploy_to_ctrl(ctx).await?;

  init_deploys(envs_dir.as_path(), &db_pool).await?;

  check_url(
    plugin_name.as_str(),
    env_id.as_str(),
    plugin_env_id.as_str(),
    &db_pool,
  )
  .await?;

  Ok(())
}

#[test_context(TenantProjectContext)]
#[tokio::test]
async fn test_deploy_plugin_online(
  ctx: &mut TenantProjectContext,
) -> Result<()> {
  let envs_dir = PathBuf::from(ctx.envs_dir());
  let db_pool = ctx.db_pool().clone();
  let env_id = ctx.proj().env_id().clone();

  let (plugin_name, plugin_env_id, plugin_req, code_req) =
    deploy_to_ctrl(ctx).await?;

  add_plugin_deploy(
    plugin_name.as_str(),
    envs_dir.as_path(),
    plugin_env_id.as_str(),
    plugin_req.deploy_seq,
    &plugin_req.codes,
    &plugin_req.http_routes,
  )
  .await?;

  add_code_deploy(
    envs_dir.as_path(),
    env_id.as_str(),
    code_req.deploy_seq,
    &code_req.codes,
    &code_req.http_routes,
  )
  .await?;

  check_url(
    plugin_name.as_str(),
    env_id.as_str(),
    plugin_env_id.as_str(),
    &db_pool,
  )
  .await?;
  Ok(())
}

async fn deploy_to_ctrl(
  ctx: &mut TenantProjectContext,
) -> Result<(String, String, AddCodeDeployReq, AddCodeDeployReq)> {
  let db_pool = &ctx.db_pool().clone();
  let env_id = ctx.proj().env_id().clone();
  let _envs_dir = PathBuf::from(ctx.envs_dir());
  let txn = db_pool
    .begin()
    .await
    .context("Failed to start transaction")?;

  let codes = vec![Code {
    fs_path: "functions/hello.js".to_string(),
    content: r#"export default function hello() {return "hi";}"#.to_string(),
  }];

  let plugin_name = format!("{}_test_plugin", new_nano_id());
  let plugin_env_id = {
    let plugin_proj =
      Project::new_plugin_proj("test_plugin_org", plugin_name.as_str());
    let plugin_env_id = plugin_proj.env_id().to_string();
    let txn = db_pool.begin().await?;
    let txn = plugin_proj.save(txn).await?;
    txn.commit().await?;
    ctx.set_plugin_proj(plugin_proj);
    plugin_env_id
  };

  println!(
    "plugin_name: {}, plugin_env_id: {}",
    plugin_name.as_str(),
    plugin_env_id.as_str()
  );

  let (deploy_seq, codes, routes, txn) =
    deploy_plugin(txn, plugin_env_id.as_str(), plugin_name.as_str(), &codes)
      .await?;
  let plugin_req = AddCodeDeployReq {
    env_id: plugin_env_id.clone(),
    deploy_seq,
    codes: codes.clone(),
    http_routes: routes.clone(),
  };

  let (deploy_seq, codes, routes, txn) =
    deploy_code(txn, env_id.as_str(), &codes, &None, &None).await?;
  let code_req = AddCodeDeployReq {
    env_id: env_id.clone(),
    deploy_seq,
    codes: codes.clone(),
    http_routes: routes.clone(),
  };

  txn.commit().await.context("Failed to commit transaction")?;
  Ok((plugin_name, plugin_env_id, plugin_req, code_req))
}

async fn check_url(
  plugin_name: &str,
  env_id: &str,
  plugin_env_id: &str,
  db_pool: &MySqlPool,
) -> Result<()> {
  let plugin_hello_url = format!("_plugins/{}/hello", plugin_name);

  let (ret_env_id, _seq, r) =
    match_route(env_id, plugin_hello_url.as_str(), "POST")
      .expect("should match schema plugin url");
  assert_eq!(ret_env_id.as_str(), plugin_env_id);
  assert_eq!(r.http_path, "hello");
  assert_eq!(r.js_entry_point, "functions/hello.js");
  assert_eq!(r.js_export, "default");

  let http_routes = list_api(&db_pool, env_id).await?;
  assert_eq!(
    http_routes
      .iter()
      .filter(|r| { r.http_path == plugin_hello_url })
      .count(),
    1
  );
  Ok(())
}
