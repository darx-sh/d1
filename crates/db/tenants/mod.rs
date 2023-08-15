use async_trait::async_trait;
use sea_query::Iden;
use serde::Deserialize;
use std::any::Any;
use std::fmt::Write;

mod ddl;
mod dml;
mod execute;
mod pool;

pub use pool::{
  add_tenant_db_info, get_tenant_pool, test_tenant_db_info, MySqlTenantPool,
  TenantDBInfo,
};

#[async_trait]
pub trait TenantConnPool {
  async fn js_execute(
    &self,
    query: &str,
    params: Vec<serde_json::Value>,
  ) -> anyhow::Result<serde_json::Value>;

  fn as_any(&self) -> &dyn Any;
}

#[derive(Deserialize)]
pub enum DxDatum {
  Bool(bool),
  Int64(i64),
  Text(String),
  Numeric(f64),
  Double(f64),
  Date(String),
  DateTime(String),
  Json(String),
  Null,
}

#[derive(Deserialize)]
pub enum DxFieldType {
  Bool,
  Int64,
  Text,
  // Fixed point
  Numeric,
  // Approximate numeric values
  Double,
  Date,
  DateTime,
  Json,
}

#[derive(Deserialize)]
pub struct DxColumnType {
  pub name: String,
  pub field_type: DxFieldType,
  pub is_nullable: bool,
}

/// DxIdent is a wrapper around a string that implements Iden,
/// it represents an identifier in a SQL query like column name, table name.
pub struct DxIdent(String);

impl Iden for DxIdent {
  fn unquoted(&self, s: &mut dyn Write) {
    write!(s, "{}", self.0).unwrap();
  }
}

///
/// schema api: client --> data plane
///
#[derive(Deserialize)]
pub struct CreateTableReq {
  pub table_name: String,
  pub columns: Vec<DxColumnType>,
  //   todo primary key, index...
}

#[derive(Deserialize)]
pub struct DropTableReq {
  pub table_name: String,
}

#[derive(Deserialize)]
pub struct AddColumnReq {
  pub table_name: String,
  pub column: DxColumnType,
}

#[derive(Deserialize)]
pub struct DropColumnReq {
  pub table_name: String,
  pub column_name: String,
}

#[derive(Deserialize)]
pub struct RenameColumnReq {
  pub table_name: String,
  pub old_column_name: String,
  pub new_column_name: String,
}

///
/// table api: client --> data plane
/// SELECT * FROM table_name WHERE created_at >= '2021-01-01' AND id NOT IN (1344, 231243) ORDER BY created_at DESC LIMIT 100
///
#[derive(Deserialize)]
pub struct PaginationTableReq {
  pub table_name: String,
  pub prev_created_at: Option<String>,
  pub prev_ids: Option<Vec<String>>,
  pub limit: u64,
}

#[derive(Deserialize)]
pub struct InsertTableReq {
  pub table_name: String,
  pub columns: Vec<String>,
  pub values: Vec<Vec<DxDatum>>,
}

#[derive(Deserialize)]
pub struct UpdateTableReq {
  pub table_name: String,
  pub columns: Vec<ColumnValue>,
  pub filter: Option<String>,
}

#[derive(Deserialize)]
pub struct ColumnValue {
  pub name: String,
  pub value: DxDatum,
}

#[derive(Deserialize)]
pub struct DeleteTableReq {
  pub table_name: String,
}
