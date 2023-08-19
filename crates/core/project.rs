use crate::code::control::deploy_var;
use crate::env_vars::{Var, VarList};
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
  db_info: TenantDBInfo,
  default_var_list: VarList,
}

impl Project {
  pub fn new(org_id: &str, name: &str) -> Self {
    let id = new_nano_id();
    let env_id = new_nano_id();
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
      env_id.as_str(),
      &vec![Var::new("DX_DB_NAME", format!("dx_{}", env_id).as_str())],
    );

    Project {
      id,
      proj_name: name.to_string(),
      org_id: org_id.to_string(),
      env_id,
      db_info,
      default_var_list: var_list,
    }
  }

  pub fn id(&self) -> &ProjectId {
    &self.id
  }

  pub fn env_id(&self) -> &EnvId {
    &self.env_id
  }

  pub async fn save(&self, pool: &MySqlPool) -> Result<()> {
    let mut txn = pool.begin().await?;
    let mut txn = save_tenant_project(
      txn,
      self.id.as_str(),
      self.org_id.as_str(),
      self.proj_name.as_str(),
    )
    .await?;

    let mut txn =
      save_env(self.id.as_str(), self.env_id.as_str(), "dev", txn).await?;

    save_tenant_db(&mut txn, self.env_id.as_str(), &self.db_info).await?;

    let txn =
      save_default_env_vars(txn, self.env_id.as_str(), &self.default_var_list)
        .await?;
    txn.commit().await?;
    Ok(())
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
