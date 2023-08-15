use async_trait::async_trait;
use std::any::Any;

mod execute;
mod pool;

pub use pool::{
  add_tenant_db_info, get_tenant_pool, test_tenant_db_info, MySqlTenantPool,
  TenantDBInfo,
};

#[async_trait]
pub trait TenantConnPool {
  async fn js_execute(
    &self,
    query: &str,
    params: Vec<serde_json::Value>,
  ) -> anyhow::Result<serde_json::Value>;

  fn as_any(&self) -> &dyn Any;
}
