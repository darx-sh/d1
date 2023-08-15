use crate::TenantConnPool;
use anyhow::anyhow;
use dashmap::DashMap;
use once_cell::sync::Lazy;
use sqlx::mysql::MySqlConnectOptions;
use sqlx::MySqlPool;
use std::ops::{Deref, DerefMut};

pub struct TenantDBInfo {
  pub host: String,
  pub port: u16,
  pub user: String,
  pub password: String,
  pub database: String,
}

/// [`MySqlTenantPool`] represents a single tenant's connection pool.
/// It is just a simple [`MySqlPool`].
pub struct MySqlTenantPool(pub MySqlPool);

impl MySqlTenantPool {
  pub fn inner(&self) -> &MySqlPool {
    &(self.0)
  }
}

impl Deref for MySqlTenantPool {
  type Target = MySqlPool;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl DerefMut for MySqlTenantPool {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

pub fn test_tenant_db_info(env_id: &str) -> TenantDBInfo {
  TenantDBInfo {
    host: "localhost".to_string(),
    port: 3306,
    user: env_id.to_string(),
    password: env_id.to_string(),
    database: format!("dx_{}", env_id),
  }
}

static GLOBAL_POOL: Lazy<DashMap<String, MySqlPool>> = Lazy::new(DashMap::new);

static GLOBAL_DB_INFO: Lazy<DashMap<String, TenantDBInfo>> =
  Lazy::new(DashMap::new);

pub async fn get_tenant_pool(
  env_id: &str,
) -> anyhow::Result<Box<dyn TenantConnPool>> {
  if let Some(pool) = GLOBAL_POOL.get(env_id) {
    Ok(Box::new(MySqlTenantPool(pool.value().clone())))
  } else {
    let db_info = GLOBAL_DB_INFO
      .get(env_id)
      .ok_or_else(|| anyhow!("db info not found"))?;

    // todo: user per env lock to avoid duplicate connection
    let mysql_conn_options = MySqlConnectOptions::new()
      .host(&db_info.host)
      .port(db_info.port)
      .username(&db_info.user)
      .password(&db_info.password)
      .database(&db_info.database);
    let new_pool = MySqlPool::connect_with(mysql_conn_options).await?;
    let pool = GLOBAL_POOL
      .entry(env_id.to_string())
      .or_insert_with(|| new_pool.clone());
    Ok(Box::new(MySqlTenantPool(pool.value().clone())))
  }
}

pub fn add_tenant_db_info(env_id: &str, db_info: TenantDBInfo) {
  GLOBAL_DB_INFO.insert(env_id.to_string(), db_info);
}
