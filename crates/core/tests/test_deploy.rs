use anyhow::{Context, Result};
use darx_core::deploy::control::{deploy_code, list_api};
use darx_core::deploy::data::{
  add_deployment, init_deployments, invoke_function, match_route,
};
use darx_core::plugin::{deploy_system_plugins, SYS_PLUGIN_SCHEMA_ENV_ID};
use darx_core::Code;
use serde_json::json;
use serial_test::serial;
use std::env;
use std::path::PathBuf;

const TEST_ENV_ID: &str = "cljb3ovlt0002e38vwo0xi5ge";

#[serial]
#[tokio::test]
async fn test_basic() -> Result<()> {
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
    deploy_code(txn, TEST_ENV_ID, &codes, &None, &None).await?;

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
async fn test_plugin() -> Result<()> {
  let db_pool = sqlx::MySqlPool::connect(
    env::var("DATABASE_URL")
      .expect("DATABASE_URL should be configured")
      .as_str(),
  )
  .await
  .context("Failed to connect database")?;

  deploy_system_plugins(&db_pool).await?;

  //  deploy user's code
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
    deploy_code(txn, TEST_ENV_ID, &codes, &None, &None).await?;

  txn.commit().await.context("Failed to commit transaction")?;

  let envs_dir = envs_dir();
  init_deployments(envs_dir.as_path(), &db_pool).await?;

  let (env_id, seq, r) =
    match_route(TEST_ENV_ID, "_plugins/schema/create_table", "POST")
      .expect("should match schema plugin url");
  assert_eq!(env_id, SYS_PLUGIN_SCHEMA_ENV_ID);
  assert_eq!(r.http_path, "create_table");
  assert_eq!(r.js_entry_point, "functions/create_table.js");
  assert_eq!(r.js_export, "default");

  let http_routes = list_api(&db_pool, TEST_ENV_ID).await?;
  assert_eq!(http_routes.len(), 6);
  assert_eq!(
    http_routes
      .iter()
      .filter(|r| {
        r.http_path == "_plugins/schema/create_table"
          || r.http_path == "_plugins/schema/drop_table"
          || r.http_path == "_plugins/schema/rename_table"
          || r.http_path == "_plugins/schema/alter_table"
      })
      .count(),
    4
  );
  Ok(())
}

fn envs_dir() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/darx_envs")
}
