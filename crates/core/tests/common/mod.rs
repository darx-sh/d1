use crate::Project;
use anyhow::Context;
use std::env;
use std::path::{Path, PathBuf};
use test_context::AsyncTestContext;
use tokio::fs;

pub struct TenantProjectContext {
  envs_dir: PathBuf,
  proj: Project,
  db_pool: sqlx::MySqlPool,
  plugin_proj: Option<Project>,
}

impl TenantProjectContext {
  pub fn envs_dir(&self) -> &Path {
    self.envs_dir.as_path()
  }

  pub fn proj(&self) -> &Project {
    &self.proj
  }

  pub fn db_pool(&self) -> &sqlx::MySqlPool {
    &self.db_pool
  }

  pub fn set_plugin_proj(&mut self, proj: Project) {
    self.plugin_proj = Some(proj);
  }
}

#[async_trait::async_trait]
impl AsyncTestContext for TenantProjectContext {
  async fn setup() -> Self {
    let envs_dir =
      PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/darx_envs");
    let db_pool = sqlx::MySqlPool::connect(
      env::var("DATABASE_URL")
        .expect("DATABASE_URL should be configured")
        .as_str(),
    )
    .await
    .context("Failed to connect database")
    .unwrap();
    let proj = Project::new_tenant_proj("test_org", "test_proj");
    let txn = db_pool.begin().await.unwrap();
    let txn = proj.save(txn).await.unwrap();
    txn.commit().await.unwrap();
    TenantProjectContext {
      envs_dir,
      proj,
      db_pool,
      plugin_proj: None,
    }
  }

  async fn teardown(self) {
    self.proj.drop(&self.db_pool).await.unwrap();
    let _ = fs::remove_dir_all(self.envs_dir.join(self.proj.env_id())).await;

    if let Some(plugin) = self.plugin_proj {
      plugin.drop(&self.db_pool).await.unwrap();
      let _ = fs::remove_dir_all(self.envs_dir.join(plugin.env_id())).await;
    }
  }
}
