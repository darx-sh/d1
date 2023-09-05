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
  rename_column_sql, rename_table_sql,
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

#[derive(Deserialize, Debug)]
pub enum DxDefaultValue {
  #[serde(rename = "bool")]
  Bool(bool),
  #[serde(rename = "int64")]
  Int64(i64),
  #[serde(rename = "float64")]
  Float64(f64),
  #[serde(rename = "text")]
  Text(String),
  #[serde(rename = "datetime")]
  DateTime(String),
  #[serde(rename = "expr")]
  Expr(String),
  #[serde(rename = "NULL")]
  Null,
}

#[derive(Deserialize, Debug, PartialEq)]
pub enum DxFieldType {
  #[serde(rename = "bool")]
  Bool,
  #[serde(rename = "int64")]
  Int64,
  #[serde(rename = "int64_identity")]
  Int64Identity,
  #[serde(rename = "text")]
  Text,
  // Approximate numeric values
  #[serde(rename = "float64")]
  Float64,
  #[serde(rename = "datetime")]
  DateTime,
}

#[derive(Deserialize, Debug)]
pub struct DxColumnType {
  pub name: String,
  #[serde(rename = "fieldType")]
  pub field_type: DxFieldType,
  #[serde(rename = "isNullable")]
  pub is_nullable: bool,
  #[serde(rename = "defaultValue")]
  pub default_value: Option<DxDefaultValue>,
  pub extra: Option<String>,
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
  #[serde(rename = "renameTable")]
  RenameTable(RenameTableReq),
  #[serde(rename = "dropTable")]
  DropTable(DropTableReq),
  #[serde(rename = "addColumn")]
  AddColumn(AddColumnReq),
  #[serde(rename = "dropColumn")]
  DropColumn(DropColumnReq),
  #[serde(rename = "renameColumn")]
  RenameColumn(RenameColumnReq),
}

#[derive(Deserialize, Debug)]
pub struct CreateTableReq {
  #[serde(rename = "tableName")]
  pub table_name: String,
  pub columns: Vec<DxColumnType>,
  //   todo primary key, index...
}

#[derive(Deserialize, Debug)]
pub struct RenameTableReq {
  #[serde(rename = "oldTableName")]
  pub old_table_name: String,
  #[serde(rename = "newTableName")]
  pub new_table_name: String,
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
            "fieldType": "int64",
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
