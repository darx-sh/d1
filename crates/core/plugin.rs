use crate::api::ApiError;
use crate::EnvKind;
use anyhow::Result;

pub const SYS_PLUGIN_SCHEMA_API: &str = "schema";
pub const SYS_PLUGIN_TABLE_API: &str = "table";

pub fn plugin_project_id(plugin_name: &str) -> String {
  format!("000000000000_{}", plugin_name)
}

pub fn plugin_env_id(plugin_name: &str, env_kind: &EnvKind) -> String {
  format!("000000000000_{}_{}", plugin_name, env_kind.as_str())
}

// pub async fn deploy_system_plugins(db_pool: &MySqlPool) -> Result<()> {
//   // schema api plugin
//   let schema_api = include_str!("plugin_data/schema/api.js");
//   let schema_api_codes = vec![Code {
//     fs_path: "functions/api.js".to_string(),
//     content: schema_api.to_string(),
//   }];
//   deploy_plugin(
//     db_pool,
//     SYS_PLUGIN_SCHEMA_API_PROJECT_ID,
//     SYS_PLUGIN_SCHEMA_API_ENV_ID,
//     SYS_PLUGIN_SCHEMA_API_NAME,
//     &schema_api_codes,
//   )
//   .await?;
//
//   // table api plugin
//   let table_api = include_str!("plugin_data/table/api.js");
//   let table_api_codes = vec![Code {
//     fs_path: "functions/api.js".to_string(),
//     content: table_api.to_string(),
//   }];
//   deploy_plugin(
//     db_pool,
//     SYS_PLUGIN_TABLE_API_PROJECT_ID,
//     SYS_PLUGIN_TABLE_API_ENV_ID,
//     SYS_PLUGIN_TABLE_API_NAME,
//     &table_api_codes,
//   )
//   .await?;
//   Ok(())
// }

pub fn plugin_http_path(plugin_name: &str, path: &str) -> String {
  format!("_plugins/{}/{}", plugin_name, path)
}

// returns (plugin_name, url)
pub fn parse_plugin_url(func_url: &str) -> Result<(String, String)> {
  let parts: Vec<&str> = func_url.split("/").collect();
  if parts.len() < 3 {
    return Err(ApiError::InvalidPluginUrl(func_url.to_string()).into());
  }
  let plugin_name = parts[1].to_string();
  let url = parts[2..].join("/");
  Ok((plugin_name, url))
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn test_parse_plugin_url() -> Result<()> {
    let (env_id, url) = parse_plugin_url("_plugins/env1/func1")?;
    assert_eq!(env_id, "env1");
    assert_eq!(url, "func1");

    let (env_id, url) = parse_plugin_url("_plugins/env1/")?;
    assert_eq!(env_id, "env1");
    assert_eq!(url, "");

    assert_eq!(parse_plugin_url("_plugins").is_err(), true);
    assert_eq!(parse_plugin_url("_plugins/env1").is_err(), true);
    Ok(())
  }
}
