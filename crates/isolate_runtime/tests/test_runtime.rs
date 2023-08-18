use anyhow::Result;
use darx_db::{add_tenant_db_info, get_tenant_pool, TenantDBInfo};
use darx_db::{drop_tenant_db, setup_tenant_db};
use darx_isolate_runtime::{build_snapshot, DarxIsolate};
use darx_utils::test_control_db_url;
use sqlx::Connection;
use std::ops::DerefMut;
use std::path::PathBuf;
use std::time::Duration;

const TEST_ENV_ID: &str = "8nvcym53y8d2";
const TEST_DEPLOY_SEQ: i64 = 99;

pub fn test_tenant_db_info(env_id: &str) -> TenantDBInfo {
  TenantDBInfo {
    host: "localhost".to_string(),
    port: 3306,
    user: env_id.to_string(),
    password: env_id.to_string(),
    database: format!("dx_{}", env_id),
  }
}

pub fn env_deploy_path(env_id: &str, deploy_seq: i64) -> PathBuf {
  let deploy_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .join(format!("tests/data/{}/{}", env_id, deploy_seq));
  deploy_path
}

async fn env_db_setup(env_id: &str, deploy_seq: i64) -> Result<PathBuf> {
  let deploy_path = env_deploy_path(env_id, deploy_seq);
  // setup tenant db
  let mut conn =
    sqlx::mysql::MySqlConnection::connect(test_control_db_url()).await?;
  let mut txn = conn.begin().await?;
  let db_info = test_tenant_db_info(env_id);
  drop_tenant_db(txn.deref_mut(), env_id, &db_info).await?;
  setup_tenant_db(txn.deref_mut(), env_id, &db_info).await?;
  txn.commit().await?;

  add_tenant_db_info(env_id, db_info);
  Ok(deploy_path)
}

#[tokio::test]
async fn test_run() {
  let deploy_path = env_deploy_path(TEST_ENV_ID, TEST_DEPLOY_SEQ);
  let mut darx_runtime =
    DarxIsolate::new(TEST_ENV_ID, TEST_DEPLOY_SEQ, deploy_path.as_path());

  darx_runtime
    .load_and_eval_module_file("foo.js")
    .await
    .expect("foo.js should not result an error");
  darx_runtime
    .load_and_eval_module_file("bar.js")
    .await
    .expect("bar.js should not result an error");
}

#[tokio::test]
async fn test_private() {
  let deploy_path = env_deploy_path(TEST_ENV_ID, TEST_DEPLOY_SEQ);
  let mut darx_runtime =
    DarxIsolate::new(TEST_ENV_ID, TEST_DEPLOY_SEQ, deploy_path.as_path());
  let r = darx_runtime
    .load_and_eval_module_file("load_private.js")
    .await;
  assert!(r.is_err());
}

#[tokio::test]
async fn test_db_query() -> Result<()> {
  let deploy_path = env_db_setup(TEST_ENV_ID, TEST_DEPLOY_SEQ).await?;
  let pool = get_tenant_pool(TEST_ENV_ID).await?;
  pool.js_execute("DROP TABLE IF EXISTS test", vec![]).await?;
  pool
    .js_execute(
      "CREATE TABLE IF NOT EXISTS test (
            id INT NOT NULL AUTO_INCREMENT,
            name VARCHAR(255) NOT NULL,
            PRIMARY KEY (id)
        )",
      vec![],
    )
    .await?;

  let mut darx_runtime =
    DarxIsolate::new(TEST_ENV_ID, TEST_DEPLOY_SEQ, deploy_path.as_path());
  darx_runtime
    .load_and_eval_module_file("run_query.js")
    .await?;

  let mut dx_runtime =
    DarxIsolate::new(TEST_ENV_ID, TEST_DEPLOY_SEQ, deploy_path.as_path());
  dx_runtime.load_and_eval_module_file("run_ddl.js").await?;
  Ok(())
}

#[tokio::test]
async fn test_bad_db_conn() -> Result<()> {
  // This env has no db setup, so it should fail when using db connection.
  let env_id = "000000000000_schema_dev";
  let deploy_seq = 3;
  let deploy_path = env_deploy_path(env_id, deploy_seq);
  let mut darx_runtime =
    DarxIsolate::new(env_id, deploy_seq, deploy_path.as_path());

  // run registration (simulate the snapshot process)
  let module_id = darx_runtime
    .js_runtime
    .load_side_module(
      &deno_core::resolve_path("__registry.js", deploy_path.as_path())?,
      None,
    )
    .await?;
  let receiver = darx_runtime.js_runtime.mod_evaluate(module_id);
  darx_runtime.js_runtime.run_event_loop(false).await?;
  let _ = receiver.await?;

  // call the function
  let script_result = darx_runtime
    .js_runtime
    .execute_script("invoking_function", "functions_api_runQuery()")?;
  let script_result = darx_runtime.js_runtime.resolve_value(script_result);
  let duration = Duration::from_secs(5);
  let script_result = tokio::time::timeout(duration, script_result).await?;
  assert_eq!(script_result.is_err(), true);
  Ok(())
}

#[tokio::test]
#[ignore]
async fn test_db_bad_conn_snapshot() -> Result<()> {
  // This env has no db setup, so it should fail when using db connection.
  let env_id = "000000000000_schema_dev";
  let deploy_seq = 3;
  let deploy_path = env_deploy_path(env_id, deploy_seq);

  let snapshot = build_snapshot(deploy_path.as_path(), "__registry.js").await?;
  let snapshot_box = snapshot.as_ref().to_vec().into_boxed_slice();
  let mut isolate = DarxIsolate::new_with_snapshot(
    env_id,
    deploy_seq,
    deploy_path.as_path(),
    snapshot_box,
  )
  .await;

  let script_result = isolate
    .js_runtime
    .execute_script("invoking_function", "functions_api_runQuery()")?;
  let script_result = isolate.js_runtime.resolve_value(script_result);
  let duration = Duration::from_secs(5);
  let script_result = tokio::time::timeout(duration, script_result).await?;
  assert_eq!(script_result.is_err(), true);
  Ok(())
}
