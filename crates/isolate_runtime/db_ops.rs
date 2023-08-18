use crate::EnvId;
use anyhow::anyhow;
use darx_db::{
  add_column_sql, create_table_sql, drop_column_sql, drop_table_sql,
  get_tenant_pool, rename_column_sql, DDLReq, TenantConnPool,
};
use deno_core::error::AnyError;
use deno_core::{op, ResourceId};
use deno_core::{OpState, Resource};
use std::borrow::Cow;
use std::cell::RefCell;
use std::rc::Rc;

deno_core::extension!(
  darx_db_ops,
  deps = [darx_bootstrap],
  ops = [
    op_use_db,
    op_db_execute,
    op_ddl,
    // op_select_statement,
    // op_select_from,
    // op_select_columns,
    // op_select_build
  ],
  esm = ["js/01_db.js"]
);

struct ConnResource(Box<dyn TenantConnPool>);

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
  let r = get_tenant_pool(env_id.0.as_str()).await;
  match r {
    Err(e) => {
      tracing::error!("useDB error: {}", e);
      Err(anyhow!("useDB error: {}", e))
    }
    Ok(conn) => {
      let rid = op_state.borrow_mut().resource_table.add(ConnResource(conn));
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
  let conn = &conn_resource.0;
  conn.js_execute(query.as_str(), params).await
}

#[op]
pub async fn op_ddl(
  op_state: Rc<RefCell<OpState>>,
  rid: ResourceId,
  req: DDLReq,
) -> Result<serde_json::Value, AnyError> {
  let conn_resource =
    op_state.borrow().resource_table.get::<ConnResource>(rid)?;
  let conn = &conn_resource.0;
  let sql = match req {
    DDLReq::CreateTable(req) => create_table_sql(&req),
    DDLReq::DropTable(req) => drop_table_sql(&req),
    DDLReq::AddColumn(req) => add_column_sql(&req),
    DDLReq::DropColumn(req) => drop_column_sql(&req),
    DDLReq::RenameColumn(req) => rename_column_sql(&req),
  }?;
  conn.js_execute(sql.as_str(), vec![]).await
}

// struct SelectStatementResource(RefCell<SelectStatement>);
//
// impl Resource for SelectStatementResource {
//   fn name(&self) -> Cow<str> {
//     "selectStatementResource".into()
//   }
// }
//
// #[op]
// pub fn op_select_statement(
//   op_state: &mut OpState,
// ) -> Result<ResourceId, AnyError> {
//   let query = RefCell::new(Query::select());
//   let rid = op_state.resource_table.add(SelectStatementResource(query));
//   Ok(rid)
// }
//
// #[op]
// pub fn op_select_columns(
//   op_state: &mut OpState,
//   rid: ResourceId,
//   fields: Vec<String>,
// ) -> Result<(), AnyError> {
//   let mut query = op_state
//     .resource_table
//     .get::<SelectStatementResource>(rid)?;
//   let mut query = query.0.borrow_mut();
//   let fields = fields.into_iter().map(DxIdent).collect::<Vec<_>>();
//   query.columns(fields);
//   Ok(())
// }
//
// #[op]
// pub fn op_select_from(
//   op_state: &mut OpState,
//   rid: ResourceId,
//   table: String,
// ) -> Result<(), AnyError> {
//   let mut query = op_state
//     .resource_table
//     .get::<SelectStatementResource>(rid)?;
//   let mut query = query.0.borrow_mut();
//   query.from(DarxIden(table));
//   Ok(())
// }
//
// #[op]
// pub fn op_select_build(
//   op_state: &mut OpState,
//   rid: ResourceId,
// ) -> Result<(), AnyError> {
//   let query = op_state
//     .resource_table
//     .get::<SelectStatementResource>(rid)?;
//   let query = query.0.borrow();
//   let query = query.to_string(MysqlQueryBuilder);
//   println!("select build: {}", query);
//   Ok(())
// }
