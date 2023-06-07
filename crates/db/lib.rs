pub mod mysql;

use crate::mysql::MySqlPool;
use anyhow::Result;
use async_trait::async_trait;
use serde::Serialize;
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
