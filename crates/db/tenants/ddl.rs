use crate::tenants::{
  AddColumnReq, CreateTableReq, DropColumnReq, DropTableReq, DxDefaultValue,
  RenameColumnReq, RenameTableReq,
};
use crate::tenants::{DxColumnType, DxFieldType, DxIdent};
use anyhow::Result;
use sea_query::{ColumnDef, Expr, Index, MysqlQueryBuilder, Table};
// use sqlx::MySqlExecutor;

// pub async fn create_table<'c>(
//   exe: impl MySqlExecutor<'c>,
//   req: &CreateTableReq,
// ) -> Result<()> {
//   let sql = create_table_sql(req)?;
//   sqlx::query(&sql).execute(exe).await?;
//   Ok(())
// }
//
// pub async fn drop_table<'c>(
//   exe: impl MySqlExecutor<'c>,
//   req: &DropTableReq,
// ) -> Result<()> {
//   let sql = drop_table_sql(req)?;
//   sqlx::query(&sql).execute(exe).await?;
//   Ok(())
// }
//
// pub async fn add_column<'c>(
//   exe: impl MySqlExecutor<'c>,
//   req: &AddColumnReq,
// ) -> Result<()> {
//   let sql = add_column_sql(req)?;
//   sqlx::query(&sql).execute(exe).await?;
//   Ok(())
// }
//
// pub async fn drop_column<'c>(
//   exe: impl MySqlExecutor<'c>,
//   req: &DropColumnReq,
// ) -> Result<()> {
//   let sql = drop_column_sql(req)?;
//   sqlx::query(&sql).execute(exe).await?;
//   Ok(())
// }
//
// pub async fn rename_column<'c>(
//   exe: impl MySqlExecutor<'c>,
//   req: &RenameColumnReq,
// ) -> Result<()> {
//   let sql = rename_column_sql(req)?;
//   sqlx::query(&sql).execute(exe).await?;
//   Ok(())
// }

pub fn create_table_sql(req: &CreateTableReq) -> Result<String> {
  println!("create_table_sql: {:?}", req);

  let mut stmt = Table::create();

  // add default columns and indexes
  // let mut id = ColumnDef::new(DxIdent("id".to_string()));
  // id.string().string_len(255);
  // id.not_null();
  //
  // // CURRENT_TIMESTAMP(3) is not supported by sea_query
  // let mut created_at = ColumnDef::new(DxIdent("created_at".to_string()));
  // created_at.custom(DxIdent("datetime(3)".to_string()));
  // created_at.not_null();
  // created_at.default(Expr::cust("CURRENT_TIMESTAMP(3)"));
  //
  // let mut updated_at = ColumnDef::new(DxIdent("updated_at".to_string()));
  // updated_at.custom(DxIdent("datetime(3)".to_string()));
  // updated_at.not_null();
  // updated_at.default(Expr::cust("CURRENT_TIMESTAMP(3)"));
  // updated_at.extra("ON UPDATE CURRENT_TIMESTAMP(3)");

  stmt.table(DxIdent(req.table_name.clone()));
  // stmt.col(&mut id).col(&mut created_at).col(&mut updated_at);
  for column in &req.columns {
    let mut column_def = new_column_def(column);
    stmt.col(&mut column_def);
  }

  // todo: send it from frontend.
  stmt.primary_key(Index::create().col(DxIdent("id".to_string())));
  stmt.index(
    Index::create()
      .name(format!("idx_{}_{}", req.table_name, "created_at"))
      .col(DxIdent("created_at".to_string())),
  );
  Ok(stmt.build(MysqlQueryBuilder))
}

pub fn rename_table_sql(req: &RenameTableReq) -> Result<String> {
  let mut stmt = Table::rename();
  stmt.table(
    DxIdent(req.old_table_name.clone()),
    DxIdent(req.new_table_name.clone()),
  );
  Ok(stmt.build(MysqlQueryBuilder))
}

pub fn drop_table_sql(req: &DropTableReq) -> Result<String> {
  let mut stmt = Table::drop();
  stmt.table(DxIdent(req.table_name.clone()));
  Ok(stmt.build(MysqlQueryBuilder))
}

pub fn add_column_sql(req: &AddColumnReq) -> Result<String> {
  let mut stmt = Table::alter();
  stmt.table(DxIdent(req.table_name.clone()));
  let mut column_def = new_column_def(&req.column);
  stmt.add_column(&mut column_def);
  Ok(stmt.build(MysqlQueryBuilder))
}

pub fn rename_column_sql(req: &RenameColumnReq) -> Result<String> {
  let mut stmt = Table::alter();
  stmt.table(DxIdent(req.table_name.clone()));
  stmt.rename_column(
    DxIdent(req.old_column_name.clone()),
    DxIdent(req.new_column_name.clone()),
  );
  Ok(stmt.build(MysqlQueryBuilder))
}

pub fn drop_column_sql(req: &DropColumnReq) -> Result<String> {
  let mut stmt = Table::alter();
  stmt.table(DxIdent(req.table_name.clone()));
  stmt.drop_column(DxIdent(req.column_name.clone()));
  Ok(stmt.build(MysqlQueryBuilder))
}

fn new_column_def(column_type: &DxColumnType) -> ColumnDef {
  let mut column_def = ColumnDef::new(DxIdent(column_type.name.clone()));
  match column_type.field_type {
    DxFieldType::Bool => column_def.boolean(),
    DxFieldType::Int64 => column_def.big_integer(),
    DxFieldType::Text => column_def.text(),
    DxFieldType::Float64 => column_def.double(),
    DxFieldType::DateTime => {
      column_def.custom(DxIdent("datetime(3)".to_string()))
    }
  };

  if column_type.is_nullable {
    column_def.null();
  } else {
    column_def.not_null();
  }

  if let Some(default_value) = &column_type.default_value {
    match default_value {
      DxDefaultValue::Bool(v) => {
        column_def.default(*v);
      }
      DxDefaultValue::Int64(v) => {
        column_def.default(*v);
      }
      DxDefaultValue::Float64(v) => {
        column_def.default(*v);
      }
      DxDefaultValue::Text(v) => {
        column_def.default(v);
      }
      DxDefaultValue::DateTime(v) => {
        column_def.default(v);
      }
      DxDefaultValue::Expr(v) => {
        column_def.default(Expr::cust(v.clone()));
      }
      DxDefaultValue::Null => {
        // do nothing
      }
    }
  }

  if let Some(extra) = &column_type.extra {
    column_def.extra(extra);
  }
  column_def
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_create_table() {
    let req = CreateTableReq {
      table_name: "test".to_string(),
      columns: vec![
        DxColumnType {
          name: "id".to_string(),
          field_type: DxFieldType::Int64,
          is_nullable: false,
          default_value: None,
          extra: None,
        },
        DxColumnType {
          name: "created_at".to_string(),
          field_type: DxFieldType::DateTime,
          is_nullable: false,
          default_value: Some(DxDefaultValue::Expr(
            "CURRENT_TIMESTAMP(3)".to_string(),
          )),
          extra: None,
        },
        DxColumnType {
          name: "updated_at".to_string(),
          field_type: DxFieldType::DateTime,
          is_nullable: false,
          default_value: Some(DxDefaultValue::Expr(
            "CURRENT_TIMESTAMP(3)".to_string(),
          )),
          extra: Some("ON UPDATE CURRENT_TIMESTAMP(3)".to_string()),
        },
        DxColumnType {
          name: "age".to_string(),
          field_type: DxFieldType::Int64,
          is_nullable: false,
          default_value: None,
          extra: None,
        },
      ],
    };

    assert_eq!(
      create_table_sql(&req).unwrap(),
      "CREATE TABLE `test` ( `id` bigint NOT NULL, `created_at` datetime(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3), `updated_at` datetime(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3), `age` bigint NOT NULL )"
    );
  }

  #[test]
  fn test_add_column() {
    let req = AddColumnReq {
      table_name: "test".to_string(),
      column: DxColumnType {
        name: "age".to_string(),
        field_type: DxFieldType::Int64,
        is_nullable: true,
        default_value: None,
        extra: None,
      },
    };

    assert_eq!(
      add_column_sql(&req).unwrap(),
      "ALTER TABLE `test` ADD COLUMN `age` bigint NULL"
    );
  }

  #[test]
  fn test_rename_column() {
    let req = RenameColumnReq {
      table_name: "test".to_string(),
      old_column_name: "id".to_string(),
      new_column_name: "id_2".to_string(),
    };

    assert_eq!(
      rename_column_sql(&req).unwrap(),
      "ALTER TABLE `test` RENAME COLUMN `id` TO `id_2`"
    );
  }

  #[test]
  fn test_drop_column() {
    let req = DropColumnReq {
      table_name: "test".to_string(),
      column_name: "age".to_string(),
    };

    assert_eq!(
      drop_column_sql(&req).unwrap(),
      "ALTER TABLE `test` DROP COLUMN `age`"
    );
  }
}
