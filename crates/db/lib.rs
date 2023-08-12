use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;

mod control;
mod tenants;

pub use tenants::{
  add_tenant_db_info, get_tenant_pool, test_tenant_db_info, TenantDBInfo,
};

pub use control::{drop_tenant_db, setup_tenant_db};

#[async_trait]
pub trait TenantConnPool {
  async fn execute(
    &self,
    query: &str,
    params: Vec<serde_json::Value>,
  ) -> Result<serde_json::Value>;
}
