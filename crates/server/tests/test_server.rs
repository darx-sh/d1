use anyhow::Result;
use darx_core::api::{
  DeployVarReq, NewPluginProjectReq, NewProjectRsp, NewTenantProjectReq,
};
use darx_utils::new_nano_id;
use dotenv::dotenv;
use serde_json::json;
use std::collections::HashMap;
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

const DATA: &str = "127.0.0.1:3456";
const CONTROL: &str = "127.0.0.1:3457";

#[actix_web::test]
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

  // prepare test data
  let project_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  let code_path = project_dir.join("tests/basic_test/user_home/alice");
  let server_data_path = project_dir.join("tests/basic_test/server");
  _ = tokio::fs::remove_dir_all(&server_data_path).await;
  tokio::fs::create_dir(&server_data_path).await.unwrap();

  let handle = run_server(server_data_path).await;

  // create tenant project
  let req = NewTenantProjectReq {
    org_id: "test_org".to_string(),
    project_name: "test_proj".to_string(),
  };
  let client = reqwest::Client::new();
  let rsp = client
    .post(format!("http://{}/new_tenant_project", CONTROL))
    .json(&req)
    .send()
    .await
    .unwrap()
    .error_for_status()
    .unwrap()
    .json::<NewProjectRsp>()
    .await
    .unwrap();
  let env_id = rsp.env_id;

  // create plugin project
  let plugin_name = format!("{}_test_plugin", new_nano_id());
  let req = NewPluginProjectReq {
    org_id: "test_org".to_string(),
    plugin_name: plugin_name.clone(),
  };
  let client = reqwest::Client::new();
  let rsp = client
    .post(format!("http://{}/new_plugin_project", CONTROL))
    .json(&req)
    .send()
    .await
    .unwrap()
    .error_for_status()
    .unwrap()
    .json::<NewProjectRsp>()
    .await
    .unwrap();
  let _plugin_env_id = rsp.env_id;

  // deploy_var
  let mut vars = HashMap::new();
  vars.insert("key1".to_string(), "value1".to_string());
  let req = DeployVarReq { desc: None, vars };
  let client = reqwest::Client::new();
  client
    .post(format!("http://{}/deploy_var/{}", CONTROL, env_id.as_str()))
    .json(&req)
    .send()
    .await
    .unwrap()
    .error_for_status()
    .unwrap();

  // deploy_code
  let req = darx_core::api::dir_to_deploy_code_req(code_path.as_path())
    .await
    .unwrap();
  info!("req: {:#?}", req);
  let client = reqwest::Client::new();

  client
    .post(format!(
      "http://{}/deploy_code/{}",
      CONTROL,
      env_id.as_str()
    ))
    .json(&req)
    .send()
    .await
    .unwrap()
    .error_for_status()
    .unwrap();

  let req = json!({"msg": "123"});
  let resp = client
    .post(format!("http://{}/invoke/foo.Hi", DATA))
    .header("Darx-Dev-Host", format!("{}.darx.sh", env_id.as_str()))
    .json(&req)
    .send()
    .await
    .unwrap()
    .error_for_status()
    .unwrap()
    .text()
    .await
    .unwrap();
  assert_eq!(resp, "\"Hi 123 from foo, env key1 = value1\"");

  let req = json!({"arr": [1,2,3], "obj":{"msg":"obj"}, "num": 1});

  let resp = client
    .post(format!("http://{}/invoke/bar.Hi", DATA))
    .header("Darx-Dev-Host", format!("{}.darx.sh", env_id.as_str()))
    .json(&req)
    .send()
    .await
    .unwrap()
    .error_for_status()
    .unwrap()
    .text()
    .await
    .unwrap();
  assert_eq!(resp, "\"Hi 1 obj 1 null from bar\"");

  // deploy plugin
  let code_path =
    project_dir.join("tests/basic_test/user_home/plugins/test_plugin");
  let req = darx_core::api::dir_to_deploy_plugin_req(
    plugin_name.as_str(),
    code_path.as_path(),
  )
  .await
  .unwrap();
  let client = reqwest::Client::new();
  client
    .post(format!("http://{}/deploy_plugin", CONTROL))
    .json(&req)
    .send()
    .await
    .unwrap()
    .error_for_status()
    .unwrap();

  let plugin_api_url = format!(
    "http://{}/invoke/_plugins/{}/api.listTable",
    DATA, plugin_name
  );
  let status = client
    .post(plugin_api_url)
    .header("Darx-Dev-Host", format!("{}.darx.sh", env_id))
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
