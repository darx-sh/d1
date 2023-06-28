use crate::types::{MySqlQueryResult, XDatum};
use crate::{DeployId, EnvId, ProjectId};
use darx_db::get_conn;
use darx_db::{Connection, ConnectionPool};
use deno_core::error::AnyError;
use deno_core::{op, ResourceId};
use deno_core::{OpState, Resource};
use std::borrow::Cow;
use std::cell::RefCell;
use std::future::Future;
use std::mem;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::Arc;

deno_core::extension!(
    darx_db_ops,
    deps = [darx_bootstrap],
    ops = [op_use_db, op_db_execute],
    esm = ["js/01_db.js"]
);

struct ConnResource(Rc<RefCell<dyn Connection>>);

impl Resource for ConnResource {
    fn name(&self) -> Cow<str> {
        "connResource".into()
    }
}

#[op]
pub async fn op_use_db(
    op_state: Rc<RefCell<OpState>>,
) -> Result<ResourceId, AnyError> {
    let env_id = op_state.borrow().borrow::<EnvId>().clone();
    let deploy_id = op_state.borrow().borrow::<DeployId>().clone();

    let r = get_conn(env_id.0.as_str(), deploy_id.0.as_str()).await;
    match r {
        Err(e) => {
            println!("useDB error: {}", e);
            Err(e)
        }
        Ok(conn) => {
            let rid =
                op_state.borrow_mut().resource_table.add(ConnResource(conn));
            println!("rust rid = {}", rid);
            Ok(rid)
        }
    }
}

#[op]
pub async fn op_db_execute(
    op_state: Rc<RefCell<OpState>>,
    rid: ResourceId,
    query: String,
    params: Vec<serde_json::Value>,
) -> Result<serde_json::Value, AnyError> {
    let conn_resource = op_state
        .borrow_mut()
        .resource_table
        .get::<ConnResource>(rid)?;
    let mut conn = conn_resource.0.borrow_mut();
    conn.execute(query.as_str(), params).await
}
