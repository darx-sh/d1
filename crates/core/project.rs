use crate::code::control::deploy_var;
use crate::env_vars::{Var, VarList};
use anyhow::{Context, Result};
use darx_db::setup_tenant_db;
use darx_db::TenantDBInfo;
use darx_utils::new_nano_id;
use sqlx::{MySql, MySqlPool, Transaction};
use std::env;
use std::ops::DerefMut;

pub async fn new_tenant_project(
  pool: &MySqlPool,
  org_id: &str,
  proj_name: &str,
) -> Result<(String, String)> {
  let project_id = new_nano_id();
  let mut txn = pool.begin().await?;

  sqlx::query!(
    "INSERT INTO `projects` (`id`, `org_id`, `name`) VALUES (?, ?, ?)",
    project_id,
    org_id,
    proj_name
  )
  .execute(txn.deref_mut())
  .await
  .context("Failed to insert into projects table")?;

  let (env_id, txn) = new_env(project_id.as_str(), "dev", txn).await?;
  let txn = new_env_db(txn, env_id.as_str()).await?;
  let txn = set_default_env_vars(txn, env_id.as_str()).await?;
  txn.commit().await?;
  Ok((project_id, env_id))
}

async fn new_env<'c>(
  project_id: &str,
  env_name: &str,
  mut txn: Transaction<'c, MySql>,
) -> Result<(String, Transaction<'c, MySql>)> {
  let env_id = new_nano_id();

  sqlx::query!(
    "INSERT INTO `envs` (`id`, `project_id`, `name`) VALUES (?, ?, ?)",
    env_id,
    project_id,
    env_name
  )
  .execute(txn.deref_mut())
  .await
  .context("Failed to insert into envs table")?;
  Ok((env_id, txn))
}

async fn new_env_db<'c>(
  mut txn: Transaction<'c, MySql>,
  env_id: &str,
) -> Result<Transaction<'c, MySql>> {
  let db_host =
    env::var("DATA_PLANE_DB_HOST").expect("DATA_PLANE_DB_HOST not set");
  let db_port =
    env::var("DATA_PLANE_DB_PORT").expect("DATA_PLANE_DB_PORT not set");
  let db_user = env_id;
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
  setup_tenant_db(&mut txn, env_id, &db_info).await?;
  Ok(txn)
}

async fn set_default_env_vars<'c>(
  mut txn: Transaction<'c, MySql>,
  env_id: &str,
) -> Result<Transaction<'c, MySql>> {
  let var_list = VarList::new_env_vars(
    env_id,
    &vec![Var::new("DX_DB_NAME", format!("dx_{}", env_id).as_str())],
  );

  var_list.save(txn.deref_mut()).await?;
  let (_, _, txn) = deploy_var(
    txn,
    env_id,
    &vec![],
    &Some("default env deploy".to_string()),
  )
  .await?;
  Ok(txn)
}
