use crate::types::{MySqlQueryResult, XDatum};
use deno_core::error::AnyError;
use deno_core::op;
use deno_core::OpState;
use mysql_async::prelude::Queryable;
use std::cell::RefCell;
use std::rc::Rc;

deno_core::extension!(
    darx_db,
    deps = [darx_bootstrap],
    ops = [op_db_query],
    esm = ["js/01_db.js"]
);

#[op]
pub async fn op_db_query(
    state: Rc<RefCell<OpState>>,
    query_str: String,
    params: Vec<serde_json::Value>,
) -> Result<MySqlQueryResult, AnyError> {
    let pool = state.borrow().borrow::<mysql_async::Pool>().clone();
    let mut conn = pool.get_conn().await?;
    let params: Vec<XDatum> = params.into_iter().map(|v| v.into()).collect();

    let mut query_result = conn.exec_iter(query_str, params).await?;
    let query_result = if query_result.is_empty() {
        MySqlQueryResult::Empty {
            affected_rows: query_result.affected_rows(),
            last_insert_id: query_result.last_insert_id(),
        }
    } else {
        let xrows = query_result.map(|r| r.into()).await?;
        MySqlQueryResult::Rows(xrows)
    };
    Ok(query_result)
}

pub fn create_db_pool() -> mysql_async::Pool {
    mysql_async::Pool::new("mysql://root:12345678@localhost:3306/test")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::DarxRuntime;
    use deno_core::anyhow::Result;
    use mysql_async::prelude::Query;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_db_query() -> Result<()> {
        let pool = create_db_pool();
        let mut conn = pool.get_conn().await?;
        r"CREATE TABLE IF NOT EXISTS test (
            id INT NOT NULL AUTO_INCREMENT, 
            name VARCHAR(255) NOT NULL, 
            PRIMARY KEY (id)
        )"
        .ignore(&mut conn)
        .await?;
        let tenant_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("examples/tenants/7ce52fdc14b16017");
        let mut darx_runtime = DarxRuntime::new(pool, tenant_path);
        darx_runtime.run("run_query.js").await?;
        Ok(())
    }
}
