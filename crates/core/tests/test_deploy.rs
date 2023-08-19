use anyhow::{Context, Result};
use darx_core::code::control::{
  deploy_code, deploy_plugin, deploy_var, list_api,
};
use darx_core::env_vars::Var;
use darx_core::tenants::{
  add_code_deploy, add_var_deploy, init_deploys, invoke_function, match_route,
};
use darx_core::Code;
use serde_json::json;
use serial_test::serial;
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;

const TEST_ENV_ID: &str = "8nvcym53y8d2";

#[serial]
#[tokio::test]
async fn test_deploy_code() -> Result<()> {
  let db_pool = sqlx::MySqlPool::connect(
    env::var("DATABASE_URL")
      .expect("DATABASE_URL should be configured")
      .as_str(),
  )
  .await
  .context("Failed to connect database")?;

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
    deploy_code(txn, TEST_ENV_ID, &codes, &None, &None).await?;

  let (var_deploy_seq, vars, txn) =
    deploy_var(txn, TEST_ENV_ID, &vec![Var::new("key1", "value1")], &None)
      .await?;

  txn.commit().await.context("Failed to commit transaction")?;

  let envs_dir = envs_dir();
  add_code_deploy(
    envs_dir.as_path(),
    TEST_ENV_ID,
    code_deploy_seq,
    &final_codes,
    &http_routes,
  )
  .await?;

  let vars: HashMap<_, _> = vars
    .into_iter()
    .map(|item| (item.key().to_string(), item.val().to_string()))
    .collect();
  add_var_deploy(TEST_ENV_ID, var_deploy_seq, &vars).await?;

  let (env_id, seq, r) =
    match_route(TEST_ENV_ID, "hello", "POST").expect("should match url");
  assert_eq!(env_id, TEST_ENV_ID);
  assert_eq!(code_deploy_seq, seq);

  let ret = invoke_function(
    envs_dir.as_path(),
    env_id.as_str(),
    seq,
    json!({}),
    &r.js_entry_point,
    &r.js_export,
    &r.func_sig.param_names,
  )
  .await?;
  assert_eq!(ret, json!("hi"));

  let (env_id, seq, r) =
    match_route(TEST_ENV_ID, "hello2", "POST").expect("should match url");
  assert_eq!(code_deploy_seq, seq);
  assert_eq!(env_id, TEST_ENV_ID);

  let ret = invoke_function(
    envs_dir.as_path(),
    env_id.as_str(),
    seq,
    json!({}),
    &r.js_entry_point,
    &r.js_export,
    &r.func_sig.param_names,
  )
  .await?;
  assert_eq!(ret, json!("hi2"));

  let (env_id, seq, r) =
    match_route(TEST_ENV_ID, "getEnvNone", "POST").expect("should match url");
  assert_eq!(code_deploy_seq, seq);
  assert_eq!(env_id, TEST_ENV_ID);
  let ret = invoke_function(
    envs_dir.as_path(),
    env_id.as_str(),
    seq,
    json!({}),
    &r.js_entry_point,
    &r.js_export,
    &r.func_sig.param_names,
  )
  .await?;
  assert_eq!(ret, serde_json::Value::Null);

  let (env_id, seq, r) =
    match_route(TEST_ENV_ID, "getEnvKey1", "POST").expect("should match url");
  assert_eq!(code_deploy_seq, seq);
  assert_eq!(env_id, TEST_ENV_ID);
  let ret = invoke_function(
    envs_dir.as_path(),
    env_id.as_str(),
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

#[tokio::test]
#[serial]
async fn test_deploy_plugin() -> Result<()> {
  let db_pool = sqlx::MySqlPool::connect(
    env::var("DATABASE_URL")
      .expect("DATABASE_URL should be configured")
      .as_str(),
  )
  .await
  .context("Failed to connect database")?;

  let txn = db_pool
    .begin()
    .await
    .context("Failed to start transaction")?;

  let codes = vec![Code {
    fs_path: "functions/hello.js".to_string(),
    content: r#"export default function hello() {return "hi";}"#.to_string(),
  }];

  let (_, _, _, txn) =
    deploy_plugin(txn, "0000_test_plugin", "test_plugin", &codes).await?;
  let (_, _, _, txn) =
    deploy_code(txn, TEST_ENV_ID, &codes, &None, &None).await?;

  txn.commit().await.context("Failed to commit transaction")?;

  let envs_dir = envs_dir();
  init_deploys(envs_dir.as_path(), &db_pool).await?;

  let plugin_hello = "_plugins/test_plugin/hello";

  let (env_id, _seq, r) = match_route(TEST_ENV_ID, plugin_hello, "POST")
    .expect("should match schema plugin url");
  assert_eq!(env_id, "0000_test_plugin");
  assert_eq!(r.http_path, "hello");
  assert_eq!(r.js_entry_point, "functions/hello.js");
  assert_eq!(r.js_export, "default");

  let http_routes = list_api(&db_pool, TEST_ENV_ID).await?;
  assert_eq!(
    http_routes
      .iter()
      .filter(|r| { r.http_path == plugin_hello })
      .count(),
    1
  );
  Ok(())
}

fn envs_dir() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/darx_envs")
}
