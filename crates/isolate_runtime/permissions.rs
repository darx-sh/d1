use deno_core::error::AnyError;
use deno_core::url::Url;
use std::path::{Path, PathBuf};

pub struct Permissions {
    _project_dir: PathBuf,
}

impl Permissions {
    pub fn new(options: Options) -> Self {
        Self {
            _project_dir: options.bundle_dir,
        }
    }
}

pub struct Options {
    pub bundle_dir: PathBuf,
}

deno_core::extension!(
    darx_permissions,
    options = {
        options: Options,
    },
    state = |state, options| {
        state.put::<Permissions>(Permissions::new(options.options));
    }
);

impl deno_fetch::FetchPermissions for Permissions {
    fn check_net_url(
        &mut self,
        _url: &Url,
        _api_name: &str,
    ) -> Result<(), AnyError> {
        Ok(())
    }

    fn check_read(
        &mut self,
        _p: &Path,
        _api_name: &str,
    ) -> Result<(), AnyError> {
        Ok(())
    }
}

impl deno_web::TimersPermission for Permissions {
    fn allow_hrtime(&mut self) -> bool {
        true
    }

    fn check_unstable(
        &self,
        _state: &deno_core::OpState,
        _api_name: &'static str,
    ) {
    }
}
