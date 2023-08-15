use anyhow::{Context, Result};
use darx_db::setup_tenant_db;
use darx_db::TenantDBInfo;
use darx_utils::new_nano_id;
use sqlx::{Executor, MySqlConnection, MySqlPool};
use std::env;
use std::ops::DerefMut;

pub async fn new_project(
  pool: &MySqlPool,
  org_id: &str,
  proj_name: &str,
) -> Result<(String, String)> {
  let project_id = new_nano_id();
  let mut txn = pool.begin().await?;
  let conn = txn.deref_mut();
  conn
    .execute(sqlx::query!(
      "INSERT INTO `projects` (`id`, `org_id`, `name`) VALUES (?, ?, ?)",
      project_id,
      org_id,
      proj_name
    ))
    .await
    .context("Failed ton insert into projects table")?;

  let env_id = new_env(project_id.as_str(), "dev", conn).await?;
  new_env_db(&mut txn, env_id.as_str()).await?;
  txn.commit().await?;
  Ok((project_id, env_id))
}

async fn new_env<'c>(
  project_id: &str,
  env_name: &str,
  conn: &mut MySqlConnection,
) -> Result<String> {
  let env_id = new_nano_id();

  sqlx::query!(
    "INSERT INTO `envs` (`id`, `project_id`, `name`) VALUES (?, ?, ?)",
    env_id,
    project_id,
    env_name
  )
  .execute(conn)
  .await
  .context("Failed to insert into envs table")?;
  Ok(env_id)
}

async fn new_env_db<'c>(
  conn: &mut MySqlConnection,
  env_id: &str,
) -> Result<()> {
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
  setup_tenant_db(conn, env_id, &db_info).await
}
