use crate::tenants::{DxIdent, PaginationTableReq};
use anyhow::Result;
use sea_query::{Asterisk, Expr, MysqlQueryBuilder, Order, Query};

fn paginationTableSql(
  req: &PaginationTableReq,
) -> Result<(String, sea_query::Values)> {
  let mut query = Query::select();
  query.column(Asterisk).from(DxIdent(req.table_name.clone()));

  if let Some(prev_created_at) = &req.prev_created_at {
    query.and_where(
      Expr::col(DxIdent("create_at".to_string())).gte(prev_created_at),
    );
  }

  if let Some(prev_ids) = &req.prev_ids {
    query.and_where(Expr::col(DxIdent("id".to_string())).is_not_in(prev_ids));
  }
  query.order_by(DxIdent("created_at".to_string()), Order::Desc); // DESC
  query.limit(req.limit);
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
    let (sql, values) = paginationTableSql(&req).unwrap();
    assert_eq!(
      sql,
      "SELECT * FROM `test` ORDER BY `created_at` DESC LIMIT ?"
    );
    assert_eq!(values, Values(vec![Value::BigUnsigned(Some(100))]));
  }
}
