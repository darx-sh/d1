use std::ops::DerefMut;
use std::path::PathBuf;

use anyhow::Result;
use darx_db::{
  add_tenant_db_info, get_tenant_pool, test_tenant_db_info, TenantConnPool,
};
use darx_db::{drop_tenant_db, setup_tenant_db};
use sqlx::Connection;

use darx_isolate_runtime::DarxIsolate;
use darx_utils::test_control_db_url;

const TEST_ENV_ID: &str = "cljb3ovlt0002e38vwo0xi5ge";

pub fn isolate_input() -> (String, i64, PathBuf) {
  let deploy_seq = 99;
  let deploy_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .join(format!("tests/data/{}/{}", TEST_ENV_ID, deploy_seq));
  (TEST_ENV_ID.to_string(), deploy_seq, deploy_path)
}

#[tokio::test]
async fn test_run() {
  let (env_id, deploy_seq, deploy_path) = isolate_input();
  let mut darx_runtime =
    DarxIsolate::new(env_id.as_str(), deploy_seq, deploy_path.as_path());

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
  let (env_id, deploy_seq, deploy_path) = isolate_input();

  let mut darx_runtime =
    DarxIsolate::new(env_id.as_str(), deploy_seq, deploy_path.as_path());
  let r = darx_runtime
    .load_and_eval_module_file("load_private.js")
    .await;
  assert!(r.is_err());
}

#[tokio::test]
async fn test_db_query() -> Result<()> {
  // let pool = sqlx::mysql::MySqlPool::connect(test_mysql_db_url()).await?;
  // sqlx::query(
  //   "CREATE TABLE IF NOT EXISTS test (
  //           id INT NOT NULL AUTO_INCREMENT,
  //           name VARCHAR(255) NOT NULL,
  //           PRIMARY KEY (id)
  //       )",
  // )
  // .execute(&pool)
  // .await?;
  let (env_id, deploy_seq, deploy_path) = isolate_input();

  // setup tenant db
  let mut conn =
    sqlx::mysql::MySqlConnection::connect(test_control_db_url()).await?;
  let mut txn = conn.begin().await?;
  let db_info = test_tenant_db_info(env_id.as_str());
  drop_tenant_db(txn.deref_mut(), env_id.as_str(), &db_info).await?;
  setup_tenant_db(txn.deref_mut(), env_id.as_str(), &db_info).await?;
  txn.commit().await?;

  add_tenant_db_info(env_id.as_str(), db_info);
  let pool = get_tenant_pool(env_id.as_str()).await?;
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
    DarxIsolate::new(env_id.as_str(), deploy_seq, deploy_path.as_path());
  darx_runtime
    .load_and_eval_module_file("run_query.js")
    .await?;
  Ok(())
}
