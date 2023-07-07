use crate::{DeploySeq, EnvId};
use darx_db::get_conn;
use darx_db::Connection;
use deno_core::error::AnyError;
use deno_core::{op, ResourceId};
use deno_core::{OpState, Resource};
use std::borrow::Cow;
use std::cell::RefCell;
use std::rc::Rc;

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
    let deploy_seq = op_state.borrow().borrow::<DeploySeq>().clone();

    let r = get_conn(env_id.0.as_str(), deploy_seq.0).await;
    match r {
        Err(e) => {
            tracing::error!("useDB error: {}", e);
            Err(e)
        }
        Ok(conn) => {
            let rid =
                op_state.borrow_mut().resource_table.add(ConnResource(conn));
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
