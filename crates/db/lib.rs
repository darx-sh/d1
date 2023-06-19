pub mod mysql;
pub mod sqlite;

use crate::mysql::MySqlPool;
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::rc::Rc;

#[async_trait]
pub trait ConnectionPool: Send + Sync {
    async fn get_conn(&self) -> Result<Rc<RefCell<dyn Connection>>>;
}

#[async_trait]
pub trait Connection {
    async fn execute(
        &mut self,
        query: &str,
        params: Vec<serde_json::Value>,
    ) -> Result<serde_json::Value>;
}

pub async fn get_conn(project_id: &str) -> Result<Rc<RefCell<dyn Connection>>> {
    // todo: use per project_id cache for connection pool.
    let pool = MySqlPool::new("mysql://root:12345678@localhost:3306/test");
    let conn = pool.get_conn().await?;
    Ok(conn)
}

#[derive(Deserialize)]
pub enum DBType {
    MySQL,
    Sqlite,
}

pub fn get_db_type(project_id: &str) -> Result<DBType> {
    Ok(DBType::MySQL)
}

#[derive(Deserialize)]
pub struct Migration {
    file_name: String,
    sql: String,
}

pub type DeploymentId = u64;

#[derive(Serialize, Deserialize)]
pub struct Bundle {
    pub path: String,
    pub code: String,
}

#[derive(Serialize)]
pub enum DeploymentType {
    Schema,
    Functions,
}

#[derive(Serialize)]
pub enum DeploymentStatus {
    Doing,
    Done,
    Failed,
}

pub enum DBMigrationStatus {
    Doing,
    Done,
    Failed,
}