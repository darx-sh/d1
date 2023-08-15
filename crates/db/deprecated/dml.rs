use crate::tenants::{DxIdent, InsertTableReq, PaginationTableReq};
use anyhow::Result;
use sea_query::{Asterisk, Expr, MysqlQueryBuilder, Order, Query};

///
/// table api: client --> data plane
/// SELECT * FROM table_name WHERE created_at >= '2021-01-01' AND id NOT IN (1344, 231243) ORDER BY created_at DESC LIMIT 100
///
// #[derive(Deserialize)]
// pub struct PaginationTableReq {
//   pub table_name: String,
//   pub prev_created_at: Option<String>,
//   pub prev_ids: Option<Vec<String>>,
//   pub limit: u64,
// }
//
// #[derive(Deserialize)]
// pub struct InsertTableReq {
//   pub table_name: String,
//   pub columns: Vec<String>,
//   pub values: Vec<Vec<serde_json::Value>>,
// }
//
// #[derive(Deserialize)]
// pub struct UpdateTableReq {
//   pub table_name: String,
//   pub columns: Vec<ColumnValue>,
//   pub filter: Option<String>,
// }
//
// #[derive(Deserialize)]
// pub struct ColumnValue {
//   pub name: String,
//   pub value: DxDatum,
// }
//
// #[derive(Deserialize)]
// pub struct DeleteTableReq {
//   pub table_name: String,
// }

pub fn pagination_table_sql(
  req: &PaginationTableReq,
) -> Result<(String, sea_query::Values)> {
  let mut query = Query::select();
  query.column(Asterisk).from(DxIdent(req.table_name.clone()));

  if let Some(prev_created_at) = &req.prev_created_at {
    query.and_where(
      Expr::col(DxIdent("created_at".to_string())).gte(prev_created_at),
    );
  }

  if let Some(prev_ids) = &req.prev_ids {
    query.and_where(Expr::col(DxIdent("id".to_string())).is_not_in(prev_ids));
  }
  query.order_by(DxIdent("created_at".to_string()), Order::Desc); // DESC
  query.limit(req.limit);
  Ok(query.build(MysqlQueryBuilder))
}

pub fn insert_table_sql(
  req: &InsertTableReq,
) -> Result<(String, sea_query::Values)> {
  let mut query = Query::insert();
  query.into_table(DxIdent(req.table_name.clone()));
  query.columns(
    req
      .columns
      .iter()
      .map(|c| DxIdent(c.to_string()))
      .collect::<Vec<_>>(),
  );
  query.values_panic(req.values.clone());
  Ok(query.build(MysqlQueryBuilder))
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::tenants::PaginationTableReq;
  use sea_query::{Value, Values};

  #[test]
  fn test_pagination() {
    let req = PaginationTableReq {
      table_name: "test".to_string(),
      prev_ids: None,
      prev_created_at: None,
      limit: 100,
    };
    let (sql, values) = pagination_table_sql(&req).unwrap();
    assert_eq!(
      sql,
      "SELECT * FROM `test` ORDER BY `created_at` DESC LIMIT ?"
    );
    assert_eq!(values, Values(vec![Value::BigUnsigned(Some(100))]));

    let req = PaginationTableReq {
      table_name: "test".to_string(),
      prev_ids: Some(vec!["111".to_string(), "222".to_string()]),
      prev_created_at: Some("2023-08-15T11:53:02.247".to_string()),
      limit: 100,
    };
    let (sql, values) = pagination_table_sql(&req).unwrap();
    assert_eq!(
      sql,
      "SELECT * FROM `test` WHERE `created_at` >= ? AND `id` NOT IN (?, ?) ORDER BY `created_at` DESC LIMIT ?"
    );
    assert_eq!(
      values,
      Values(vec![
        Value::String(Some(Box::new("2023-08-15T11:53:02.247".to_string()))),
        Value::String(Some(Box::new("111".to_string()))),
        Value::String(Some(Box::new("222".to_string()))),
        Value::BigUnsigned(Some(100))
      ])
    );
  }
}
