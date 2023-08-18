use anyhow::{Context, Result};
use darx_core::code::control::{deploy_code, deploy_plugin, list_api};
use darx_core::tenants::{
  add_deployment, init_deployments, invoke_function, match_route,
};
use darx_core::Code;
use serde_json::json;
use serial_test::serial;
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
  ];

  let (deploy_seq, final_codes, http_routes, txn) =
    deploy_code(txn, TEST_ENV_ID, &codes, &vec![], &None, &None).await?;

  txn.commit().await.context("Failed to commit transaction")?;

  let envs_dir = envs_dir();
  add_deployment(
    envs_dir.as_path(),
    TEST_ENV_ID,
    deploy_seq,
    &final_codes,
    &http_routes,
  )
  .await?;

  let (env_id, seq, r) =
    match_route(TEST_ENV_ID, "hello", "POST").expect("should match url");
  assert_eq!(env_id, TEST_ENV_ID);
  assert_eq!(deploy_seq, seq);

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
  assert_eq!(deploy_seq, seq);
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
    deploy_code(txn, TEST_ENV_ID, &codes, &vec![], &None, &None).await?;

  txn.commit().await.context("Failed to commit transaction")?;

  let envs_dir = envs_dir();
  init_deployments(envs_dir.as_path(), &db_pool).await?;

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
