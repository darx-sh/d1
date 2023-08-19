use crate::tenants::TenantDBInfo;
use anyhow::{Context, Result};
use sqlx::{Executor, MySqlConnection};

pub async fn save_tenant_db(
  conn: &mut MySqlConnection,
  env_id: &str,
  db_info: &TenantDBInfo,
) -> Result<()> {
  conn.execute(sqlx::query!("\
      INSERT INTO `env_dbs` (`env_id`, `db_type`, `db_host`, `db_port`, `db_user`, `db_password`, `db_name`) \
      VALUES (?, ?, ?, ?, ?, ?, ?)",
      env_id,
      "mysql",
      db_info.host,
      db_info.port,
      db_info.user,
      db_info.password,
      db_info.database))
    .await
    .context("Failed to insert into env_dbs table")?;

  let user_sql = format!(
    "CREATE USER IF NOT EXISTS '{}'@'%' IDENTIFIED BY '{}'",
    db_info.user, db_info.password
  );
  conn
    .execute(user_sql.as_str())
    .await
    .context("Failed to create tenant user")?;

  let db_sql = format!("CREATE DATABASE IF NOT EXISTS `{}`", db_info.database);
  conn
    .execute(db_sql.as_str())
    .await
    .context("Failed to create tenant db")?;

  let grant_sql = format!(
    "GRANT ALL PRIVILEGES ON `{}`.* TO '{}'@'%'",
    db_info.database, db_info.user
  );
  conn
    .execute(grant_sql.as_str())
    .await
    .context("Failed to grant privileges")?;

  let flush_sql = "FLUSH PRIVILEGES";
  conn
    .execute(flush_sql)
    .await
    .context("Failed to flush privileges")?;

  Ok(())
}

pub async fn drop_tenant_db(
  conn: &mut MySqlConnection,
  env_id: &str,
  db_info: &TenantDBInfo,
) -> Result<()> {
  let drop_sql = format!("DROP DATABASE IF EXISTS `{}`", db_info.database);
  conn
    .execute(drop_sql.as_str())
    .await
    .context("Failed to drop tenant db")?;

  let drop_user_sql = format!("DROP USER IF EXISTS '{}'@'%'", db_info.user);
  conn
    .execute(drop_user_sql.as_str())
    .await
    .context("Failed to drop tenant user")?;

  conn
    .execute(sqlx::query!(
      "DELETE FROM `env_dbs` WHERE `env_id` = ?",
      env_id
    ))
    .await
    .context("Failed to delete from env_dbs table")?;

  let flush_sql = "FLUSH PRIVILEGES";
  conn
    .execute(flush_sql)
    .await
    .context("Failed to flush privileges")?;

  Ok(())
}
