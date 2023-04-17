use deno_core::anyhow::{Context, Error};
use deno_core::include_js_files;
use deno_core::url::Url;
use runtime::{darx_main_js, DarxRuntime};
use std::path::{Path, PathBuf};
use std::rc::Rc;

fn main() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    let tenant_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples/tenants/7ce52fdc14b16017");
    let mut darx_runtime = DarxRuntime::new(tenant_path);

    if let Err(error) = runtime.block_on(darx_runtime.run("foo.js")) {
        eprintln!("foo error: {}", error);
    }

    if let Err(error) = runtime.block_on(darx_runtime.run("bar.js")) {
        eprintln!("bar error: {}", error)
    }

    if let Err(error) = runtime.block_on(darx_runtime.run("load_private.js")) {
        eprintln!("load_private error: {}", error)
    }
}
