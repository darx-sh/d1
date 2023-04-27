mod db;
mod module_loader;
mod permissions;
mod types;

use anyhow::{Context, Result};
use db::darx_db;
use module_loader::TenantModuleLoader;
use std::path::{Path, PathBuf};
use std::rc::Rc;

deno_core::extension!(darx_bootstrap, esm = ["js/00_bootstrap.js"]);

pub struct DarxIsolate {
    pub js_runtime: deno_core::JsRuntime,
    tenant_dir: PathBuf,
}

impl DarxIsolate {
    pub fn new(pool: mysql_async::Pool, tenant_dir: impl AsRef<Path>) -> Self {
        let user_agent = "darx-runtime".to_string();
        let root_cert_store = deno_tls::create_default_root_cert_store();

        let extensions = vec![
            permissions::darx_permissions::init_ops_and_esm(
                permissions::Options {
                    tenant_dir: PathBuf::from(tenant_dir.as_ref()),
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
            darx_db::init_ops_and_esm(),
        ];

        let mut js_runtime =
            deno_core::JsRuntime::new(deno_core::RuntimeOptions {
                module_loader: Some(Rc::new(TenantModuleLoader::new(
                    PathBuf::from(tenant_dir.as_ref()),
                ))),
                extensions,
                ..Default::default()
            });

        let op_state = js_runtime.op_state();
        op_state.borrow_mut().put::<mysql_async::Pool>(pool.clone());

        DarxIsolate {
            js_runtime,
            tenant_dir: PathBuf::from(tenant_dir.as_ref()),
        }
    }

    pub async fn load_and_eval_module_file(
        &mut self,
        file_path: &str,
    ) -> Result<()> {
        let module_id = self
            .js_runtime
            .load_side_module(
                &deno_core::resolve_path(file_path, self.tenant_dir.as_path())
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

    pub async fn load_and_evaluate_module_code(
        &mut self,
        file_path: &str,
        code: &str,
    ) -> Result<()> {
        let module_id = self
            .js_runtime
            .load_side_module(
                &deno_core::resolve_path(file_path, self.tenant_dir.as_path())
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
    use darx_utils::create_db_pool;
    #[tokio::test]
    async fn test_run() {
        let tenant_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("examples/tenants/7ce52fdc14b16017");
        let mut darx_runtime = DarxIsolate::new(create_db_pool(), tenant_path);

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
        let tenant_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("examples/tenants/7ce52fdc14b16017");
        let mut darx_runtime = DarxIsolate::new(create_db_pool(), tenant_path);
        let r = darx_runtime
            .load_and_eval_module_file("load_private.js")
            .await;
        assert!(r.is_err());
    }
}
