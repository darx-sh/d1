use anyhow::{Context, Result};
use darx_core::deploy::control::deploy_code;
use darx_core::deploy::data::{add_deployment, invoke_function, match_route};
use darx_core::Code;
use serde_json::json;
use std::env;
use std::path::PathBuf;

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

    let env_id = "cljb3ovlt0002e38vwo0xi5ge";

    let codes = vec![
        Code {
            fs_path: "functions/hello.js".to_string(),
            content: r#"export default function hello() {return "hi";}"#
                .to_string(),
        },
        Code {
            fs_path: "functions/hello2.js".to_string(),
            content: r#"export default function hello2() {return "hi2";}"#
                .to_string(),
        },
    ];

    let (deploy_seq, final_codes, http_routes, txn) =
        deploy_code(txn, env_id, &codes, &None, &None).await?;

    txn.commit().await.context("Failed to commit transaction")?;

    let bundles_dir =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/server");
    add_deployment(
        bundles_dir.as_path(),
        env_id,
        deploy_seq,
        &final_codes,
        &http_routes,
    )
    .await?;

    let (seq, r) =
        match_route(env_id, "hello", "POST").expect("should match url");
    assert_eq!(deploy_seq, seq);

    let ret = invoke_function(
        bundles_dir.as_path(),
        env_id,
        seq,
        json!({}),
        &r.js_entry_point,
        &r.js_export,
        &r.func_sig.param_names,
    )
    .await?;
    assert_eq!(ret, json!("hi"));

    let (seq, r) =
        match_route(env_id, "hello2", "POST").expect("should match url");
    assert_eq!(deploy_seq, seq);

    let ret = invoke_function(
        bundles_dir.as_path(),
        env_id,
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
