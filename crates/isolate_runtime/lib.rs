mod db_ops;
mod module_loader;
mod permissions;
mod types;

use anyhow::{Context, Result};
use darx_db::ConnectionPool;
use db_ops::darx_db_ops;
use module_loader::TenantModuleLoader;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::Arc;

deno_core::extension!(darx_bootstrap, esm = ["js/00_bootstrap.js"]);

pub struct DarxIsolate {
    pub js_runtime: deno_core::JsRuntime,
    project_dir: PathBuf,
}

#[derive(Clone)]
struct ProjectId(String);

impl DarxIsolate {
    pub fn new(project_id: &str, project_dir: impl AsRef<Path>) -> Self {
        let user_agent = "darx-runtime".to_string();
        let root_cert_store = deno_tls::create_default_root_cert_store();

        let extensions = vec![
            permissions::darx_permissions::init_ops_and_esm(
                permissions::Options {
                    project_dir: PathBuf::from(project_dir.as_ref()),
                },
            ),
            deno_webidl::deno_webidl::init_ops_and_esm(),
            deno_console::deno_console::init_ops_and_esm(),
            deno_url::deno_url::init_ops_and_esm(),
            deno_web::deno_web::init_ops_and_esm::<permissions::Permissions>(
                deno_web::BlobStore::default(),
                None,
            ),
            deno_fetch::deno_fetch::init_ops_and_esm::<permissions::Permissions>(
                deno_fetch::Options {
                    user_agent,
                    root_cert_store: Some(root_cert_store),
                    ..Default::default()
                },
            ),
            darx_bootstrap::init_ops_and_esm(),
            darx_db_ops::init_ops_and_esm(),
        ];

        let mut js_runtime =
            deno_core::JsRuntime::new(deno_core::RuntimeOptions {
                module_loader: Some(Rc::new(TenantModuleLoader::new(
                    PathBuf::from(project_dir.as_ref()),
                ))),
                extensions,
                ..Default::default()
            });
        js_runtime
            .op_state()
            .borrow_mut()
            .put::<ProjectId>(ProjectId(project_id.to_string()));

        DarxIsolate {
            js_runtime,
            project_dir: PathBuf::from(project_dir.as_ref()),
        }
    }

    /// Loads and evaluates a module from a file.
    /// The `file_path` is the path to the file relative to the project directory.
    pub async fn load_and_eval_module_file(
        &mut self,
        file_path: &str,
    ) -> Result<()> {
        let module_id = self
            .js_runtime
            .load_side_module(
                &deno_core::resolve_path(file_path, self.project_dir.as_path())
                    .with_context(|| {
                        format!("failed to resolve path: {}", file_path)
                    })?,
                None,
            )
            .await?;

        let receiver = self.js_runtime.mod_evaluate(module_id);
        self.js_runtime.run_event_loop(false).await?;
        receiver
            .await?
            .with_context(|| format!("Couldn't execute '{}'", file_path))
    }

    /// Loads and evaluates a module from a code string.
    /// The `file_path` does not have to be a real file path.
    pub async fn load_and_evaluate_module_code(
        &mut self,
        file_path: &str,
        code: &str,
    ) -> Result<()> {
        let module_id = self
            .js_runtime
            .load_side_module(
                &deno_core::resolve_path(file_path, self.project_dir.as_path())
                    .with_context(|| {
                        format!("failed to resolve path: {}", file_path)
                    })?,
                Some(code.to_string().into()),
            )
            .await?;

        let receiver = self.js_runtime.mod_evaluate(module_id);
        self.js_runtime.run_event_loop(false).await?;
        receiver
            .await?
            .with_context(|| format!("Couldn't execute '{}'", file_path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use darx_db::mysql::MySqlPool;
    use darx_utils::test_mysql_db_pool;
    #[tokio::test]
    async fn test_run() {
        let project_id = "7ce52fdc14b16017";
        let project_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(format!("examples/projects/{}", project_id));
        let conn_pool = test_mysql_db_pool();
        let mut darx_runtime = DarxIsolate::new(project_id, project_path);

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
        let project_id = "7ce52fdc14b16017";

        let project_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(format!("examples/projects/{}", project_id));
        let conn_pool = test_mysql_db_pool();

        let mut darx_runtime = DarxIsolate::new(project_id, project_dir);
        let r = darx_runtime
            .load_and_eval_module_file("load_private.js")
            .await;
        assert!(r.is_err());
    }
}
