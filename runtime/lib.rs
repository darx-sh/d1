use crate::module_loader::TenantModuleLoader;
use deno_core::anyhow::Error;
use std::path::PathBuf;
use std::rc::Rc;

mod db;
mod module_loader;
mod permissions;

deno_core::extension!(darx_main_js, esm = ["js/bootstrap.js"]);

pub struct DarxRuntime {
    js_runtime: deno_core::JsRuntime,
    tenant_dir: PathBuf,
}

impl DarxRuntime {
    pub fn new(tenant_dir: PathBuf) -> Self {
        let user_agent = "darx-runtime".to_string();
        let root_cert_store = deno_tls::create_default_root_cert_store();

        let extensions = vec![
            permissions::darx_permissions::init_ops_and_esm(
                permissions::Options {
                    tenant_dir: tenant_dir.clone(),
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
                    user_agent: user_agent.clone(),
                    root_cert_store: Some(root_cert_store.clone()),
                    ..Default::default()
                },
            ),
            darx_main_js::init_ops_and_esm(),
        ];

        let js_runtime = deno_core::JsRuntime::new(deno_core::RuntimeOptions {
            module_loader: Some(Rc::new(TenantModuleLoader::new(
                tenant_dir.clone(),
            ))),
            extensions,
            ..Default::default()
        });

        DarxRuntime {
            js_runtime,
            tenant_dir,
        }
    }

    pub async fn run(&mut self, file: &str) -> Result<(), Error> {
        let module_id = self
            .js_runtime
            .load_side_module(
                &deno_core::resolve_path(file, self.tenant_dir.as_path())
                    .unwrap(),
                None,
            )
            .await?;

        let result = self.js_runtime.mod_evaluate(module_id);
        self.js_runtime.run_event_loop(false).await?;
        let r = result.await?;
        println!("result: {:?}", r);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_run() {
        let tenant_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("examples/tenants/7ce52fdc14b16017");
        let mut darx_runtime = DarxRuntime::new(tenant_path);

        darx_runtime
            .run("foo.js")
            .await
            .expect("foo.js should not result an error");
        darx_runtime
            .run("bar.js")
            .await
            .expect("bar.js should not result an error");
    }

    #[tokio::test]
    async fn test_private() {
        let tenant_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("examples/tenants/7ce52fdc14b16017");
        let mut darx_runtime = DarxRuntime::new(tenant_path);
        let r = darx_runtime.run("load_private.js").await;
        assert!(r.is_err());
    }
}
