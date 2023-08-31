use async_trait::async_trait;
use sea_query::Iden;
use serde::Deserialize;
use std::any::Any;
use std::fmt::Write;

mod ddl;
mod execute;
mod pool;

pub use ddl::{
  add_column_sql, create_table_sql, drop_column_sql, drop_table_sql,
  rename_column_sql,
};
pub use pool::{
  add_tenant_db_info, get_tenant_pool, MySqlTenantPool, TenantDBInfo,
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

#[derive(Deserialize, Debug, PartialEq)]
pub enum DxFieldType {
  #[serde(rename = "bool")]
  Bool,
  #[serde(rename = "int64")]
  Int64,
  #[serde(rename = "text")]
  Text,
  // Approximate numeric values
  #[serde(rename = "float64")]
  Double,
  #[serde(rename = "datetime")]
  DateTime,
}

#[derive(Deserialize)]
pub struct DxColumnType {
  pub name: String,
  #[serde(rename = "fieldType")]
  pub field_type: DxFieldType,
  #[serde(rename = "isNullable")]
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
pub enum DDLReq {
  #[serde(rename = "createTable")]
  CreateTable(CreateTableReq),
  #[serde(rename = "dropTable")]
  DropTable(DropTableReq),
  #[serde(rename = "addColumn")]
  AddColumn(AddColumnReq),
  #[serde(rename = "dropColumn")]
  DropColumn(DropColumnReq),
  #[serde(rename = "renameColumn")]
  RenameColumn(RenameColumnReq),
}

#[derive(Deserialize)]
pub struct CreateTableReq {
  #[serde(rename = "tableName")]
  pub table_name: String,
  pub columns: Vec<DxColumnType>,
  //   todo primary key, index...
}

#[derive(Deserialize)]
pub struct DropTableReq {
  #[serde(rename = "tableName")]
  pub table_name: String,
}

#[derive(Deserialize)]
pub struct AddColumnReq {
  #[serde(rename = "tableName")]
  pub table_name: String,
  pub column: DxColumnType,
}

#[derive(Deserialize)]
pub struct DropColumnReq {
  #[serde(rename = "tableName")]
  pub table_name: String,
  #[serde(rename = "columnName")]
  pub column_name: String,
}

#[derive(Deserialize)]
pub struct RenameColumnReq {
  #[serde(rename = "tableName")]
  pub table_name: String,
  #[serde(rename = "oldColumnName")]
  pub old_column_name: String,
  #[serde(rename = "newColumnName")]
  pub new_column_name: String,
}

#[cfg(test)]
mod tests {
  #[test]
  fn test_des_ddl() {
    let req = r#"{
      "createTable": {
        "tableName": "test",
        "columns": [
          {
            "name": "id",
            "fieldType": "Int64",
            "isNullable": false
          }
        ]
      }
    }"#;

    let req: super::DDLReq = serde_json::from_str(req).unwrap();
    match req {
      super::DDLReq::CreateTable(req) => {
        assert_eq!(req.table_name, "test");
        assert_eq!(req.columns.len(), 1);
        assert_eq!(req.columns[0].name, "id");
        assert_eq!(req.columns[0].field_type, super::DxFieldType::Int64);
        assert_eq!(req.columns[0].is_nullable, false);
      }
      _ => panic!("expect CreateTable"),
    }
  }
}
