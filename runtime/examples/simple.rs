use runtime::db::create_db_pool;
use runtime::DarxRuntime;
use std::path::Path;

fn main() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    let tenant_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples/tenants/7ce52fdc14b16017");
    let mut darx_runtime = DarxRuntime::new(create_db_pool(), tenant_path);

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
