use anyhow::Result;
use dotenv::dotenv;
use serde_json::json;
use std::env;
use std::path::PathBuf;
use tokio::task::JoinHandle;
use tokio::time;
use tokio::time::sleep;
use tracing::info;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Layer;

const ENV_ID: &str = "8nvcym53y8d2";
const DATA: &str = "127.0.0.1:3456";
const CONTROL: &str = "127.0.0.1:3457";

#[actix_web::test]
#[ignore]
async fn test_main_process() {
  env::set_var("DATA_PLANE_URL", format!("http://{}", DATA));
  dotenv().ok();

  let registry = tracing_subscriber::registry();
  registry
    .with(
      tracing_subscriber::fmt::layer()
        .with_file(true)
        .with_line_number(true)
        .with_filter(
          tracing_subscriber::EnvFilter::builder()
            .with_default_directive(LevelFilter::INFO.into())
            .from_env_lossy(),
        ),
    )
    .init();

  let project_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  let code_path = project_dir.join("tests/basic_test");
  let server_data_path = project_dir.join("tests/basic_test/server");

  _ = tokio::fs::remove_dir_all(&server_data_path).await;
  tokio::fs::create_dir(&server_data_path).await.unwrap();

  let handle = run_server(server_data_path).await;

  // let vars = vec![darx_core::env_vars::Var::new("key", "value")];

  let req = darx_core::api::dir_to_deploy_code_req(code_path.as_path())
    .await
    .unwrap();
  info!("req: {:#?}", req);
  let client = reqwest::Client::new();

  client
    .post(format!("http://{}/deploy_code/{}", CONTROL, ENV_ID))
    .json(&req)
    .send()
    .await
    .unwrap()
    .error_for_status()
    .unwrap();

  let req = json!({"msg": "123"});

  // let resp = client
  //   .post(format!("http://{}/invoke/foo.Hi", DATA))
  //   .header("Darx-Dev-Host", format!("{}.darx.sh", ENV_ID))
  //   .json(&req)
  //   .send()
  //   .await
  //   .unwrap()
  //   .error_for_status()
  //   .unwrap()
  //   .text()
  //   .await
  //   .unwrap();
  // assert_eq!(
  //   resp,
  //   "\"Hi 123 from foo, env key = value, test_key = test_value\""
  // );
  //
  // let req = json!({"arr": [1,2,3], "obj":{"msg":"obj"}, "num": 1});
  //
  // let resp = client
  //   .post(format!("http://{}/invoke/bar.Hi", DATA))
  //   .header("Darx-Dev-Host", format!("{}.darx.sh", ENV_ID))
  //   .json(&req)
  //   .send()
  //   .await
  //   .unwrap()
  //   .error_for_status()
  //   .unwrap()
  //   .text()
  //   .await
  //   .unwrap();
  // assert_eq!(resp, "\"Hi 1 obj 1 null from bar\"");

  let status = client
    .post(format!(
      "http://{}/invoke/_plugins/schema/api.listTable",
      DATA
    ))
    .header("Darx-Dev-Host", format!("{}.darx.sh", ENV_ID))
    .json(&json!({}))
    .send()
    .await
    .unwrap()
    .error_for_status();
  assert_eq!(status.is_ok(), true);

  handle.abort();
  let _ = handle.await;
}

async fn run_server(server_data_path: PathBuf) -> JoinHandle<Result<()>> {
  let h = actix_web::rt::spawn(async {
    let data =
      darx_data_plane::run_server(DATA.parse().unwrap(), server_data_path)
        .await?;
    let control =
      darx_control_plane::run_server(CONTROL.parse().unwrap()).await?;
    let (_, _) = futures::future::try_join(data, control).await?;
    Ok(())
  });

  let client = reqwest::Client::new();

  loop {
    if client
      .get(format!("http://{}/", CONTROL))
      .send()
      .await
      .is_ok()
    {
      info!("connected {}", CONTROL);
      break;
    } else {
      sleep(time::Duration::from_secs(1)).await;
    }
  }

  loop {
    if client.get(format!("http://{}/", DATA)).send().await.is_ok() {
      info!("connected {}", DATA);
      break;
    } else {
      sleep(time::Duration::from_secs(1)).await;
    }
  }

  h
}
