use crate::api::ApiError;
use crate::deploy::control::deploy_plugin;
use crate::Code;
use anyhow::Result;
use sqlx::MySqlPool;

/// The "schema" plugin's "project_id" and "plugin_name".
const SYS_PLUGIN_SCHEMA_PROJECT_ID: &str = "pid_plugin";
pub const SYS_PLUGIN_SCHEMA_ENV_ID: &str = "000000000000_schema";
const SYS_PLUGIN_SCHEMA_NAME: &str = "schema";

pub async fn deploy_system_plugins(db_pool: &MySqlPool) -> Result<()> {
    // schema plugin
    let create_table = include_str!("plugin_data/schema/create_table.js");
    let alter_table = include_str!("plugin_data/schema/alter_table.js");
    let drop_table = include_str!("plugin_data/schema/drop_table.js");
    let rename_table = include_str!("plugin_data/schema/rename_table.js");

    let schema_codes = vec![
        Code {
            fs_path: "functions/create_table.js".to_string(),
            content: create_table.to_string(),
        },
        Code {
            fs_path: "functions/alter_table.js".to_string(),
            content: alter_table.to_string(),
        },
        Code {
            fs_path: "functions/drop_table.js".to_string(),
            content: drop_table.to_string(),
        },
        Code {
            fs_path: "functions/rename_table.js".to_string(),
            content: rename_table.to_string(),
        },
    ];

    deploy_plugin(
        db_pool,
        SYS_PLUGIN_SCHEMA_PROJECT_ID,
        SYS_PLUGIN_SCHEMA_ENV_ID,
        SYS_PLUGIN_SCHEMA_NAME,
        &schema_codes,
    )
    .await?;

    Ok(())
}

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