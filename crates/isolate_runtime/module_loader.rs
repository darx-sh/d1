use deno_core::anyhow::{anyhow, Error};
use deno_core::{
  resolve_import, ModuleLoader, ModuleSourceFuture, ModuleSpecifier,
  ResolutionKind,
};
use std::path::PathBuf;
use std::pin::Pin;

pub struct TenantModuleLoader {
  // TODO not used right now, maybe we can delete it.
  tenant_dir: PathBuf,
  fs_module_loader: deno_core::FsModuleLoader,
}

impl TenantModuleLoader {
  pub fn new(tenant_dir: PathBuf) -> Self {
    Self {
      tenant_dir,
      fs_module_loader: deno_core::FsModuleLoader,
    }
  }
}

impl ModuleLoader for TenantModuleLoader {
  fn resolve(
    &self,
    specifier: &str,
    referrer: &str,
    _kind: ResolutionKind,
  ) -> Result<ModuleSpecifier, Error> {
    let module_specifier = resolve_import(specifier, referrer)?;
    let path = module_specifier
      .to_file_path()
      .map_err(|_| anyhow!("Only file:// URLs are supported."))?;

    if path.starts_with(self.tenant_dir.as_path()) {
      Ok(module_specifier)
    } else {
      Err(anyhow!("Not allowed"))
    }
  }

  fn load(
    &self,
    module_specifier: &ModuleSpecifier,
    _maybe_referrer: Option<ModuleSpecifier>,
    _is_dyn_import: bool,
  ) -> Pin<Box<ModuleSourceFuture>> {
    self.fs_module_loader.load(
      module_specifier,
      _maybe_referrer,
      _is_dyn_import,
    )
  }
}
