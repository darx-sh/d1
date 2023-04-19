use darx_types::{XColumn, XDatum, XRow, XValue};
use deno_core::error::AnyError;
use deno_core::op;
use deno_core::OpState;
use mysql_async::prelude::WithParams;
use mysql_async::prelude::{Query, Queryable};
use mysql_async::Column;
use serde::{Deserialize, Serialize};
use serde_json;
use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

deno_core::extension!(
    darx_db,
    deps = [darx_bootstrap],
    ops = [op_db_fetch_all, op_db_exec],
    esm = ["js/01_db.js"]
);

#[op]
pub async fn op_db_fetch_all(
    state: Rc<RefCell<OpState>>,
    query_str: String,
    params: Vec<serde_json::Value>,
) -> Result<Vec<XRow>, AnyError> {
    let pool = state.borrow().borrow::<mysql_async::Pool>().clone();
    let mut conn = pool.get_conn().await?;
    let params: Vec<XDatum> = params.into_iter().map(|v| v.into()).collect();
    let query = query_str.with(params);
    let v: Vec<mysql_async::Row> = query.fetch(&mut conn).await?;
    let mut rows = vec![];
    for row in v {
        let mut xrow = XRow(vec![]);
        let columns = row.columns_ref();
        for i in 0..columns.len() {
            let cname = columns[i].name_str();
            let ty = columns[i].column_type();
            let datum: XDatum = row.as_ref(i).unwrap().into();
            let column = XColumn {
                name: cname.to_string(),
                value: XValue { datum, ty },
            };
            xrow.0.push(column);
        }
        rows.push(xrow);
    }
    Ok(rows)
}

#[op]
pub async fn op_db_exec(
    state: Rc<RefCell<OpState>>,
    query_str: String,
    params: Vec<serde_json::Value>,
) -> Result<MySqlExecResult, AnyError> {
    let pool = state.borrow().borrow::<mysql_async::Pool>().clone();
    let mut conn = pool.get_conn().await?;
    let params: Vec<XDatum> = params.into_iter().map(|v| v.into()).collect();
    // let query = query_str.with(params);

    let result = conn.exec_iter(query_str, params).await?;
    Ok(MySqlExecResult {
        rows_affected: result.affected_rows(),
        last_insert_id: result.last_insert_id().unwrap_or(0),
    })
}

#[derive(Serialize)]
pub struct MySqlExecResult {
    rows_affected: u64,
    last_insert_id: u64,
}

// impl From<MySqlQueryResult> for MySqlExecResult {
//     fn from(result: MySqlQueryResult) -> Self {
//         Self {
//             rows_affected: result.rows_affected(),
//             last_insert_id: result.last_insert_id(),
//         }
//     }
// }

/// JS (serde_json::Value) -> Rust (DarxColumnValue) -> SQLx::query::bind<T: Encode + Type<MySql>>()
/// SQLx::try_get<Decode + Type<MySql>> -> Rust (DarxColumnValue) -> JS (serde_json::Value)
// #[derive(Serialize, Deserialize)]
// struct DarxColumnValue(serde_json::Value);
//
// #[derive(Serialize, Deserialize)]
// struct DarxRow(HashMap<String, DarxColumnValue>);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{create_db_pool, DarxRuntime};
    use deno_core::anyhow::Result;
    use deno_core::futures::TryStreamExt;
    use mysql_async::prelude::Query;
    use serde_json;
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
