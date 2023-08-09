use anyhow::Result;

pub fn new_project(db_pool: &sqlx::MySqlPool) -> Result<()> {
  todo!()
}

fn new_env(txn: &mut sqlx::Transaction<'_, sqlx::MySql>) -> Result<()> {
  todo!()
}

fn new_env_db(
  txn: &mut sqlx::Transaction<'_, sqlx::MySql>,
  env_id: &str,
) -> Result<()> {
  todo!()
}
