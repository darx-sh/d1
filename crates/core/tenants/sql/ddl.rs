use crate::api::{
  AddColumnReq, CreateTableReq, DropColumnReq, DropTableReq, RenameColumnReq,
};
use crate::tenants::sql::{DxColumnType, DxFieldType, DxIdent};
use anyhow::Result;
use sea_query::{ColumnDef, MysqlQueryBuilder, Table};
use sqlx::MySqlExecutor;

pub async fn create_table<'c>(
  exe: impl MySqlExecutor<'c>,
  req: &CreateTableReq,
) -> Result<()> {
  let sql = create_table_sql(req)?;
  sqlx::query(&sql).execute(exe).await?;
  Ok(())
}

pub async fn drop_table<'c>(
  exe: impl MySqlExecutor<'c>,
  req: &DropTableReq,
) -> Result<()> {
  let sql = drop_table_sql(req)?;
  sqlx::query(&sql).execute(exe).await?;
  Ok(())
}

pub async fn add_column<'c>(
  exe: impl MySqlExecutor<'c>,
  req: &AddColumnReq,
) -> Result<()> {
  let sql = add_column_sql(req)?;
  sqlx::query(&sql).execute(exe).await?;
  Ok(())
}

pub async fn drop_column<'c>(
  exe: impl MySqlExecutor<'c>,
  req: &DropColumnReq,
) -> Result<()> {
  let sql = drop_column_sql(req)?;
  sqlx::query(&sql).execute(exe).await?;
  Ok(())
}

pub async fn rename_column<'c>(
  exe: impl MySqlExecutor<'c>,
  req: &RenameColumnReq,
) -> Result<()> {
  let sql = rename_column_sql(req)?;
  sqlx::query(&sql).execute(exe).await?;
  Ok(())
}

fn create_table_sql(req: &CreateTableReq) -> Result<String> {
  let mut stmt = Table::create();
  stmt.table(DxIdent(req.table_name.clone()));
  for column in &req.columns {
    let mut column_def = new_column_def(column);
    stmt.col(&mut column_def);
  }
  Ok(stmt.build(MysqlQueryBuilder))
}

fn drop_table_sql(req: &DropTableReq) -> Result<String> {
  let mut stmt = Table::drop();
  stmt.table(DxIdent(req.table_name.clone()));
  Ok(stmt.build(MysqlQueryBuilder))
}

fn add_column_sql(req: &AddColumnReq) -> Result<String> {
  let mut stmt = Table::alter();
  stmt.table(DxIdent(req.table_name.clone()));
  let mut column_def = new_column_def(&req.column);
  stmt.add_column(&mut column_def);
  Ok(stmt.build(MysqlQueryBuilder))
}

fn rename_column_sql(req: &RenameColumnReq) -> Result<String> {
  let mut stmt = Table::alter();
  stmt.table(DxIdent(req.table_name.clone()));
  stmt.rename_column(
    DxIdent(req.old_column_name.clone()),
    DxIdent(req.new_column_name.clone()),
  );
  Ok(stmt.build(MysqlQueryBuilder))
}

fn drop_column_sql(req: &DropColumnReq) -> Result<String> {
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
    DxFieldType::Numeric => column_def.decimal(),
    DxFieldType::Double => column_def.double(),
    DxFieldType::Date => column_def.date(),
    DxFieldType::DateTime => column_def.date_time(),
    DxFieldType::Json => column_def.json_binary(),
  };

  if column_type.is_nullable {
    column_def.null();
  } else {
    column_def.not_null();
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
      columns: vec![DxColumnType {
        name: "id".to_string(),
        field_type: DxFieldType::Int64,
        is_nullable: false,
      }],
    };

    assert_eq!(
      create_table_sql(&req).unwrap(),
      "CREATE TABLE `test` ( `id` bigint NOT NULL )"
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