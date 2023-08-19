use serde::{Deserialize, Serialize};

pub mod api;
pub mod code;
pub mod env_vars;
pub mod plugin;
mod project;
mod route_builder;

pub mod tenants;
pub use project::Project;

pub type OrgId = String;
pub type ProjectId = String;
pub type EnvId = String;
pub type DeployId = String;
pub type DeploySeq = i64;

#[derive(Debug, Serialize, Deserialize)]
pub enum EnvKind {
  #[serde(rename = "dev")]
  Dev,
  #[serde(rename = "prod")]
  Prod,
}

impl EnvKind {
  pub fn as_str(&self) -> &str {
    match self {
      EnvKind::Dev => "dev",
      EnvKind::Prod => "prod",
    }
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Code {
  pub fs_path: String,
  pub content: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HttpRoute {
  pub http_path: String,
  pub method: String,
  /// `js_entry_point` is used to find the js file.
  pub js_entry_point: String,
  pub js_export: String,

  pub func_sig_version: i32,
  pub func_sig: FunctionSignatureV1,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FunctionSignatureV1 {
  pub export_name: String,
  pub param_names: Vec<String>,
}

/// [`unique_js_export`] returns a unique function name
pub(crate) fn unique_js_export(
  js_entry_point: &str,
  js_export: &str,
) -> String {
  let js_entry_point =
    js_entry_point.strip_suffix(".js").unwrap_or(js_entry_point);
  let js_entry_point =
    js_entry_point.strip_suffix(".ts").unwrap_or(js_entry_point);
  let js_entry_point = js_entry_point
    .strip_suffix(".mjs")
    .unwrap_or(js_entry_point);
  let new_entry = js_entry_point.split("/").collect::<Vec<_>>().join("_");
  format!("{}_{}", new_entry, js_export)
}

pub(crate) const REGISTRY_FILE_NAME: &str = "__registry.js";

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_unique_js_export() {
    assert_eq!(unique_js_export("foo.js", "bar"), "foo_bar");
    assert_eq!(unique_js_export("foo/foo.js", "bar"), "foo_foo_bar");
  }
}
