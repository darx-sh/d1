use deno_core::anyhow::{Context, Error};
use runtime::darx_main_js;
use runtime::permissions::Permissions;
use std::path::Path;
use std::rc::Rc;

async fn run_js(file_path: &str) -> Result<(), Error> {
    let user_agent = "darx-runtime".to_string();
    let root_cert_store = deno_tls::create_default_root_cert_store();

    let extensions = vec![
        runtime::permissions::darx_permissions::init_ops_and_esm(),
        deno_webidl::deno_webidl::init_ops_and_esm(),
        deno_console::deno_console::init_ops_and_esm(),
        deno_url::deno_url::init_ops_and_esm(),
        deno_web::deno_web::init_ops_and_esm::<Permissions>(
            deno_web::BlobStore::default(),
            None,
        ),
        deno_fetch::deno_fetch::init_ops_and_esm::<Permissions>(
            deno_fetch::Options {
                user_agent: user_agent.clone(),
                root_cert_store: Some(root_cert_store.clone()),
                ..Default::default()
            },
        ),
        darx_main_js::init_ops_and_esm(),
    ];

    let mut js_runtime = deno_core::JsRuntime::new(deno_core::RuntimeOptions {
        module_loader: Some(Rc::new(deno_core::FsModuleLoader)),
        extensions,
        ..Default::default()
    });

    let js_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(file_path);
    let main_module = deno_core::resolve_path(
        &js_path.to_string_lossy(),
        &std::env::current_dir().context("Unable to get cwd")?,
    )?;
    let mod_id = js_runtime.load_main_module(&main_module, None).await?;
    let result = js_runtime.mod_evaluate(mod_id);
    js_runtime.run_event_loop(false).await?;
    result.await?
}

fn main() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    if let Err(error) = runtime.block_on(run_js("examples/simple.js")) {
        eprintln!("{}", error);
    }
}
