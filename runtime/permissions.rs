use deno_core::error::AnyError;
use deno_core::url::Url;
use std::path::Path;

pub struct Permissions;

impl Permissions {
    pub fn new() -> Self {
        Self
    }
}

deno_core::extension!(
    darx_permissions,
    state = |state| {
        state.put::<Permissions>(Permissions::new());
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
