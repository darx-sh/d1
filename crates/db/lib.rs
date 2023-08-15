mod control;
mod tenants;

pub use tenants::{MySqlTenantPool, TenantConnPool};

pub use control::{drop_tenant_db, setup_tenant_db};
pub use tenants::{
  add_tenant_db_info, get_tenant_pool, test_tenant_db_info, TenantDBInfo,
};
