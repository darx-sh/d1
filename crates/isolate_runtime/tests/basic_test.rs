use anyhow::Result;
use darx_isolate_runtime::DarxIsolate;
use darx_utils::test_mysql_db_url;
use mysql_async::prelude::Query;
use std::path::PathBuf;

pub fn isolate_inputs() -> (String, i64, PathBuf) {
    let env_id = "cljb3ovlt0002e38vwo0xi5ge";
    let deploy_seq: i64 = 99;
    let bundle_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(format!("tests/data/{}/{}", env_id, deploy_seq));
    (env_id.to_string(), deploy_seq, bundle_path)
}

#[tokio::test]
async fn test_run() {
    let (env_id, deploy_seq, bundle_path) = isolate_inputs();
    let conn_pool = mysql_async::Pool::new(test_mysql_db_url());
    let mut darx_runtime =
        DarxIsolate::new(env_id.as_str(), deploy_seq, bundle_path.as_path());

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
    let (env_id, deploy_seq, bundle_path) = isolate_inputs();

    let mut darx_runtime =
        DarxIsolate::new(env_id.as_str(), deploy_seq, bundle_path.as_path());
    let r = darx_runtime
        .load_and_eval_module_file("load_private.js")
        .await;
    assert!(r.is_err());
}

#[tokio::test]
async fn test_db_query() -> Result<()> {
    let conn_pool = mysql_async::Pool::new(test_mysql_db_url());
    let mut conn = conn_pool.get_conn().await?;
    r"CREATE TABLE IF NOT EXISTS test (
            id INT NOT NULL AUTO_INCREMENT,
            name VARCHAR(255) NOT NULL,
            PRIMARY KEY (id)
        )"
    .run(&mut conn)
    .await?;

    let (env_id, deploy_seq, bundle_path) = isolate_inputs();
    let mut darx_runtime =
        DarxIsolate::new(env_id.as_str(), deploy_seq, bundle_path.as_path());
    darx_runtime
        .load_and_eval_module_file("run_query.js")
        .await?;
    Ok(())
}
