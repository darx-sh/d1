mod common;
use anyhow::{Context, Result};
use common::TenantProjectContext;
use darx_core::code::control::{deploy_code, deploy_var};
use darx_core::tenants::{
  add_code_deploy, add_var_deploy, invoke_function, match_route,
};
use darx_core::{Code, Project};
use serde_json::json;
use std::collections::HashMap;
use test_context::test_context;

#[test_context(TenantProjectContext)]
#[tokio::test]
async fn test_deploy_code(ctx: &mut TenantProjectContext) -> Result<()> {
  let env_id = ctx.proj().env_id();
  let db_pool = ctx.db_pool();
  let envs_dir = ctx.envs_dir();

  let txn = db_pool
    .begin()
    .await
    .context("Failed to start transaction")?;

  let codes = vec![
    Code {
      fs_path: "functions/hello.js".to_string(),
      content: r#"export default function hello() {return "hi";}"#.to_string(),
    },
    Code {
      fs_path: "functions/hello2.js".to_string(),
      content: r#"export default function hello2() {return "hi2";}"#
        .to_string(),
    },
    Code {
      fs_path: "functions/getEnvNone.js".to_string(),
      content:
        r#"export default function getEnvNone() {return Dx.env.someKey}"#
          .to_string(),
    },
    Code {
      fs_path: "functions/getEnvKey1.js".to_string(),
      content: r#"export default function getEnvKey1() {return Dx.env.key1}"#
        .to_string(),
    },
  ];

  let (code_deploy_seq, final_codes, http_routes, txn) =
    deploy_code(txn, env_id, &codes, &None, &None).await?;

  let mut vars = HashMap::new();
  vars.insert("key1".to_string(), "value1".to_string());
  let (var_deploy_seq, vars, txn) =
    deploy_var(txn, env_id, &vars, &None).await?;

  txn.commit().await.context("Failed to commit transaction")?;

  add_code_deploy(
    envs_dir,
    env_id,
    code_deploy_seq,
    &final_codes,
    &http_routes,
  )
  .await?;

  add_var_deploy(env_id, var_deploy_seq, &vars).await?;

  let (ret_env_id, seq, r) =
    match_route(env_id, "hello", "POST").expect("should match url");
  assert_eq!(ret_env_id.as_str(), env_id);
  assert_eq!(code_deploy_seq, seq);

  let ret = invoke_function(
    envs_dir,
    env_id,
    ret_env_id.as_str(),
    seq,
    json!({}),
    &r.js_entry_point,
    &r.js_export,
    &r.func_sig.param_names,
  )
  .await?;
  assert_eq!(ret, json!("hi"));

  let (ret_env_id, seq, r) =
    match_route(env_id, "hello2", "POST").expect("should match url");
  assert_eq!(code_deploy_seq, seq);
  assert_eq!(ret_env_id.as_str(), env_id);

  let ret = invoke_function(
    envs_dir,
    env_id,
    ret_env_id.as_str(),
    seq,
    json!({}),
    &r.js_entry_point,
    &r.js_export,
    &r.func_sig.param_names,
  )
  .await?;
  assert_eq!(ret, json!("hi2"));

  let (ret_env_id, seq, r) =
    match_route(env_id, "getEnvNone", "POST").expect("should match url");
  assert_eq!(code_deploy_seq, seq);
  assert_eq!(ret_env_id.as_str(), env_id);
  let ret = invoke_function(
    envs_dir,
    env_id,
    ret_env_id.as_str(),
    seq,
    json!({}),
    &r.js_entry_point,
    &r.js_export,
    &r.func_sig.param_names,
  )
  .await?;
  assert_eq!(ret, serde_json::Value::Null);

  let (ret_env_id, seq, r) =
    match_route(env_id, "getEnvKey1", "POST").expect("should match url");
  assert_eq!(code_deploy_seq, seq);
  assert_eq!(ret_env_id.as_str(), env_id);
  let ret = invoke_function(
    envs_dir,
    env_id,
    ret_env_id.as_str(),
    seq,
    json!({}),
    &r.js_entry_point,
    &r.js_export,
    &r.func_sig.param_names,
  )
  .await?;
  assert_eq!(ret, json!("value1"));
  Ok(())
}
