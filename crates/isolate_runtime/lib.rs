use anyhow::{bail, Context, Result};
use db_ops::darx_db_ops;
use deno_core::{v8, Extension, Snapshot};
use module_loader::TenantModuleLoader;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

mod db_ops;
mod module_loader;
mod permissions;

deno_core::extension!(darx_bootstrap, esm = ["js/00_bootstrap.js"]);

pub struct DarxIsolate {
  pub js_runtime: deno_core::JsRuntime,
  deploy_dir: PathBuf,
}

#[derive(Clone)]
struct ProjectId(String);

#[derive(Clone)]
struct EnvId(String);

#[derive(Clone)]
struct DeploySeq(i64);

///
/// The DarxIsolate is a wrapper around [`deno_core::JsRuntime`].
/// [`env_id`] and [`deploy_seq`] may not correspond to the [`code_dir`].
/// For example, when the [`code_dir`] is a plugin's directory,
/// the env_id and deploy_seq might belongs to a tenant.
impl DarxIsolate {
  pub fn new(
    env_id: &str,
    deploy_seq: i64,
    vars: &HashMap<String, String>,
    code_dir: impl AsRef<Path>,
  ) -> Self {
    let mut js_runtime = deno_core::JsRuntime::new(deno_core::RuntimeOptions {
      module_loader: Some(Rc::new(TenantModuleLoader::new(PathBuf::from(
        code_dir.as_ref(),
      )))),
      extensions: DarxIsolate::extensions(code_dir.as_ref()),
      ..Default::default()
    });
    js_runtime
      .op_state()
      .borrow_mut()
      .put::<EnvId>(EnvId(env_id.to_string()));

    js_runtime
      .op_state()
      .borrow_mut()
      .put::<DeploySeq>(DeploySeq(deploy_seq));

    js_runtime
      .op_state()
      .borrow_mut()
      .put::<HashMap<String, String>>(vars.clone());

    DarxIsolate {
      js_runtime,
      deploy_dir: PathBuf::from(code_dir.as_ref()),
    }
  }

  pub async fn new_with_snapshot(
    env_id: &str,
    deploy_seq: i64,
    vars: &HashMap<String, String>,
    code_dir: impl AsRef<Path>,
    snapshot: Box<[u8]>,
  ) -> Self {
    let mut js_runtime = deno_core::JsRuntime::new(deno_core::RuntimeOptions {
      module_loader: Some(Rc::new(TenantModuleLoader::new(PathBuf::from(
        code_dir.as_ref(),
      )))),
      is_main: false,
      //TODO memory limit from env vars or env config
      create_params: Some(
        deno_core::v8::CreateParams::default()
          .heap_limits(0, 512 * 1024 * 1024),
      ),
      startup_snapshot: Some(Snapshot::Boxed(snapshot)),
      extensions: DarxIsolate::snapshot_extensions(code_dir.as_ref()),
      ..Default::default()
    });

    js_runtime
      .op_state()
      .borrow_mut()
      .put::<EnvId>(EnvId(env_id.to_string()));

    js_runtime
      .op_state()
      .borrow_mut()
      .put::<DeploySeq>(DeploySeq(deploy_seq));

    js_runtime
      .op_state()
      .borrow_mut()
      .put::<HashMap<String, String>>(vars.clone());

    DarxIsolate {
      js_runtime,
      deploy_dir: PathBuf::from(code_dir.as_ref()),
    }
  }

  pub async fn prepare_snapshot(
    code_dir: impl AsRef<Path>,
  ) -> Result<deno_core::JsRuntime> {
    let js_runtime = deno_core::JsRuntime::new(deno_core::RuntimeOptions {
      module_loader: Some(Rc::new(TenantModuleLoader::new(PathBuf::from(
        code_dir.as_ref(),
      )))),
      extensions: DarxIsolate::extensions(code_dir.as_ref()),
      will_snapshot: true,
      ..Default::default()
    });
    Ok(js_runtime)
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
        &deno_core::resolve_path(file_path, self.deploy_dir.as_path())
          .with_context(|| format!("Failed to resolve path: {}", file_path))?,
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
        &deno_core::resolve_path(file_path, self.deploy_dir.as_path())
          .with_context(|| format!("failed to resolve path: {}", file_path))?,
        Some(code.to_string().into()),
      )
      .await?;

    let receiver = self.js_runtime.mod_evaluate(module_id);
    self.js_runtime.run_event_loop(false).await?;
    receiver
      .await?
      .with_context(|| format!("Couldn't execute '{}'", file_path))
  }

  fn extensions(deploy_dir: impl AsRef<Path>) -> Vec<Extension> {
    let user_agent = "darx-runtime".to_string();
    let root_cert_store = deno_tls::create_default_root_cert_store();

    vec![
      permissions::darx_permissions::init_ops_and_esm(permissions::Options {
        deploy_dir: PathBuf::from(deploy_dir.as_ref()),
      }),
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
    ]
  }

  fn snapshot_extensions(deploy_dir: impl AsRef<Path>) -> Vec<Extension> {
    let user_agent = "darx-runtime".to_string();
    let root_cert_store = deno_tls::create_default_root_cert_store();

    vec![
      permissions::darx_permissions::init_ops(permissions::Options {
        deploy_dir: PathBuf::from(deploy_dir.as_ref()),
      }),
      deno_webidl::deno_webidl::init_ops(),
      deno_console::deno_console::init_ops(),
      deno_url::deno_url::init_ops(),
      deno_web::deno_web::init_ops::<permissions::Permissions>(
        deno_web::BlobStore::default(),
        None,
      ),
      deno_fetch::deno_fetch::init_ops::<permissions::Permissions>(
        deno_fetch::Options {
          user_agent,
          root_cert_store: Some(root_cert_store),
          ..Default::default()
        },
      ),
      darx_bootstrap::init_ops(),
      darx_db_ops::init_ops(),
    ]
  }
}

pub async fn build_snapshot(
  code_dir: &Path,
  registry_file_name: &str,
) -> Result<v8::StartupData> {
  let mut js_runtime = DarxIsolate::prepare_snapshot(&code_dir).await?;
  let registry_file = code_dir.join(registry_file_name);
  if !registry_file.exists() {
    tracing::error!("file {} does not exist", registry_file.to_string_lossy());
    bail!("file {} does not exist", registry_file.to_string_lossy());
  }

  let module_id = js_runtime
    .load_side_module(
      &deno_core::resolve_path(registry_file_name, &code_dir)?,
      None,
    )
    .await?;

  let receiver = js_runtime.mod_evaluate(module_id);
  js_runtime.run_event_loop(false).await?;
  let _ = receiver.await?;
  let snapshot = js_runtime.snapshot();
  Ok(snapshot)
}
