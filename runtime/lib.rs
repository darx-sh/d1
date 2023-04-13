pub mod permissions;
deno_core::extension!(darx_main_js, esm = ["js/bootstrap.js"]);
