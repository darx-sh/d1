use crate::api::{EnvInfo, ProjectInfo};
use crate::code::control::deploy_var;
use crate::env_vars::{Var, VarList};
use crate::plugin::{plugin_env_id, plugin_project_id};
use crate::{EnvId, OrgId, ProjectId};
use anyhow::{Context, Result};
use darx_db::save_tenant_db;
use darx_db::TenantDBInfo;
use darx_utils::new_nano_id;
use sqlx::{MySql, MySqlPool, Transaction};
use std::env;
use std::ops::DerefMut;

pub struct Project {
  id: ProjectId,
  proj_name: String,
  org_id: OrgId,
  env_id: EnvId,
  env_name: String,
  db_info: Option<TenantDBInfo>,
  var_list: VarList,
}

const DEFAULT_ENV_NAME: &str = "dev";

impl Project {
  pub async fn list_proj_info(
    org_id: &str,
    pool: &MySqlPool,
  ) -> Result<Vec<ProjectInfo>> {
    let projects =
      sqlx::query!("SELECT id, name FROM projects WHERE org_id = ?", org_id)
        .fetch_all(pool)
        .await?;

    let mut proj_infos = vec![];
    for proj in projects.iter() {
      let envs =
        sqlx::query!("SELECT id, name FROM envs WHERE project_id = ?", proj.id)
          .fetch_all(pool)
          .await?;

      let mut env_infos = vec![];
      for env in envs.iter() {
        env_infos.push(EnvInfo {
          id: env.id.clone(),
          name: env.name.clone(),
        });
      }

      proj_infos.push(ProjectInfo {
        id: proj.id.clone(),
        name: proj.name.clone(),
      });
    }
    Ok(proj_infos)
  }

  pub fn new_tenant_proj(org_id: &str, name: &str) -> Self {
    let mut proj = Project::new_minimal_proj(org_id, name);
    let env_id = proj.env_id();
    let db_host =
      env::var("DATA_PLANE_DB_HOST").expect("DATA_PLANE_DB_HOST not set");
    let db_port =
      env::var("DATA_PLANE_DB_PORT").expect("DATA_PLANE_DB_PORT not set");
    let db_user = env_id.clone();
    let db_name = format!("dx_{}", new_nano_id());

    // todo: encrypt this.
    let db_password = new_nano_id();
    let db_info = TenantDBInfo {
      host: db_host.clone(),
      port: db_port.parse::<u16>().expect("Failed to parse db port"),
      user: db_user.to_string(),
      password: db_password.clone(),
      database: db_name.clone(),
    };

    let var_list = VarList::new_env_vars(
      env_id,
      &vec![Var::new("DX_DB_NAME", format!("dx_{}", env_id).as_str())],
    );

    proj.db_info = Some(db_info);
    proj.var_list = var_list;
    proj
  }

  pub fn new_plugin_proj(org_id: &str, plugin_name: &str) -> Self {
    let mut proj = Project::new_minimal_proj(org_id, plugin_name);
    proj.id = plugin_project_id(plugin_name);
    proj.env_id = plugin_env_id(plugin_name);
    proj
  }

  pub fn id(&self) -> &ProjectId {
    &self.id
  }

  pub fn name(&self) -> &str {
    &self.proj_name
  }

  pub fn env_id(&self) -> &EnvId {
    &self.env_id
  }

  pub fn env_name(&self) -> &str {
    &self.env_name
  }

  pub fn db_info(&self) -> &Option<TenantDBInfo> {
    &self.db_info
  }

  pub async fn save(&self, pool: &MySqlPool) -> Result<()> {
    let txn = pool.begin().await?;
    let txn = save_tenant_project(
      txn,
      self.id.as_str(),
      self.org_id.as_str(),
      self.proj_name.as_str(),
    )
    .await?;

    let mut txn =
      save_env(self.id.as_str(), self.env_id.as_str(), "dev", txn).await?;

    if let Some(db_info) = &self.db_info {
      save_tenant_db(&mut txn, self.env_id.as_str(), db_info).await?;
    }

    let txn =
      save_default_env_vars(txn, self.env_id.as_str(), &self.var_list).await?;
    txn.commit().await?;
    Ok(())
  }

  pub async fn drop(&self, pool: &MySqlPool) -> Result<()> {
    let txn = pool.begin().await?;
    let txn = drop_env(txn, self.env_id.as_str()).await?;
    let txn = drop_tenant_project(txn, self.id.as_str()).await?;
    txn.commit().await?;
    Ok(())
  }

  fn new_minimal_proj(ord_id: &str, proj_name: &str) -> Self {
    let id = new_nano_id();
    let env_id = new_nano_id();
    Project {
      id,
      proj_name: proj_name.to_string(),
      org_id: ord_id.to_string(),
      env_id: env_id.clone(),
      env_name: DEFAULT_ENV_NAME.to_string(),
      db_info: None,
      var_list: VarList::new_env_vars(env_id.as_str(), &vec![]),
    }
  }
}

async fn save_tenant_project<'c>(
  mut txn: Transaction<'c, MySql>,
  project_id: &str,
  org_id: &str,
  proj_name: &str,
) -> Result<Transaction<'c, MySql>> {
  sqlx::query!(
    "INSERT INTO `projects` (`id`, `org_id`, `name`) VALUES (?, ?, ?)",
    project_id,
    org_id,
    proj_name
  )
  .execute(txn.deref_mut())
  .await
  .context("Failed to insert into projects table")?;
  Ok(txn)
}

async fn drop_tenant_project<'c>(
  mut txn: Transaction<'c, MySql>,
  project_id: &str,
) -> Result<Transaction<'c, MySql>> {
  sqlx::query!("DELETE FROM `projects` WHERE `id` = ?", project_id,)
    .execute(txn.deref_mut())
    .await
    .context("Failed to delete from projects table")?;
  Ok(txn)
}

async fn save_env<'c>(
  project_id: &str,
  env_id: &str,
  env_name: &str,
  mut txn: Transaction<'c, MySql>,
) -> Result<Transaction<'c, MySql>> {
  sqlx::query!(
    "INSERT INTO `envs` (`id`, `project_id`, `name`) VALUES (?, ?, ?)",
    env_id,
    project_id,
    env_name
  )
  .execute(txn.deref_mut())
  .await
  .context("Failed to insert into envs table")?;
  Ok(txn)
}

async fn drop_env<'c>(
  mut txn: Transaction<'c, MySql>,
  env_id: &str,
) -> Result<Transaction<'c, MySql>> {
  let deploys = sqlx::query!("SELECT id FROM deploys WHERE env_id = ?", env_id)
    .fetch_all(txn.deref_mut())
    .await
    .context("Failed to select from deploys table")?;

  sqlx::query!("DELETE FROM `envs` WHERE `id` = ?", env_id,)
    .execute(txn.deref_mut())
    .await
    .context("Failed to delete from envs table")?;

  sqlx::query!("DELETE FROM `env_vars` WHERE `env_id` = ?", env_id,)
    .execute(txn.deref_mut())
    .await
    .context("Failed to delete from env_vars table")?;

  // codes, deploy_vars, deploys
  for deploy in deploys.iter() {
    let deploy_id = &deploy.id;
    sqlx::query!("DELETE FROM `codes` WHERE `deploy_id` = ?", deploy_id,)
      .execute(txn.deref_mut())
      .await
      .context("Failed to delete from codes table")?;

    sqlx::query!("DELETE FROM `deploy_vars` WHERE `deploy_id` = ?", deploy_id,)
      .execute(txn.deref_mut())
      .await
      .context("Failed to delete from deploy_vars table")?;
  }

  sqlx::query!("DELETE FROM deploys WHERE env_id = ?", env_id)
    .execute(txn.deref_mut())
    .await
    .context("Failed to delete from deploys table")?;

  Ok(txn)
}

async fn save_default_env_vars<'c>(
  mut txn: Transaction<'c, MySql>,
  env_id: &str,
  var_list: &VarList,
) -> Result<Transaction<'c, MySql>> {
  var_list.save(txn.deref_mut()).await?;
  let (_, _, txn) = deploy_var(
    txn,
    env_id,
    &Default::default(),
    &Some("default env deploy".to_string()),
  )
  .await?;
  Ok(txn)
}
