use sea_query::Iden;
use serde::Deserialize;
use std::fmt::Write;

pub mod ddl;

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
