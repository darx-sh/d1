pub mod schema;

use crate::catalog::schema::{ColumnDesc, TableDesc};
use anyhow::Result;
use futures::TryStreamExt;
use sqlx::MySqlPool;
use sqlx::Row;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

const META_DB: &str = "darx_meta";
const APP_DB: &str = "darx_app";

#[derive(Debug)]
pub struct Catalog {
    // tables contains table definition.
    // The key is "schema_name.table_name".
    tables: RwLock<HashMap<String, TableDesc>>,
}

impl Catalog {
    pub fn get_table(&self, table_name: &str) -> Option<TableDesc> {
        let table_name = infer_full_table_name(table_name);
        self.tables.read().unwrap().get(&table_name).cloned()
    }
}

pub type CatalogRef = Arc<Catalog>;

pub async fn init_catalog(pool: &MySqlPool) -> Result<CatalogRef> {
    sqlx::query(format!("CREATE DATABASE IF NOT EXISTS {META_DB}").as_str())
        .execute(pool)
        .await?;

    sqlx::query(format!("CREATE DATABASE IF NOT EXISTS {APP_DB}").as_str())
        .execute(pool)
        .await?;

    sqlx::query(
        format!(
            r#"
CREATE TABLE IF NOT EXISTS {} (
    schema_name VARCHAR(255) NOT NULL,
    table_name VARCHAR(255) NOT NULL,
    PRIMARY KEY (schema_name, table_name)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci
    "#,
            full_table_name(META_DB, "user_tables")
        )
        .as_str(),
    )
    .execute(pool)
    .await?;

    let mut rows = sqlx::query(
        r#"
SELECT TABLE_NAME, COLUMN_NAME, DATA_TYPE, IS_NULLABLE FROM information_schema.COLUMNS 
    WHERE TABLE_SCHEMA = ? ORDER BY TABLE_NAME, ORDINAL_POSITION ASC
    "#
    ).bind(APP_DB).fetch(pool);

    let catalog = Catalog {
        tables: RwLock::new(HashMap::new()),
    };

    {
        let mut tables = catalog.tables.write().unwrap();

        while let Some(row) = rows.try_next().await? {
            let table_name: String = row.try_get(0)?;
            let column_name: String = row.try_get(1)?;
            let data_type: String = row.try_get(2)?;
            let is_nullable: String = row.try_get(3)?;
            let key = full_table_name(APP_DB, table_name);
            let table = match tables.get_mut(&key) {
                Some(table) => {
                    table.columns.push(ColumnDesc::new(
                        column_name,
                        data_type,
                        is_nullable,
                    )?);
                    table.clone()
                }
                None => TableDesc {
                    columns: vec![ColumnDesc::new(
                        column_name,
                        data_type,
                        is_nullable,
                    )?],
                },
            };
            tables.insert(key, table);
        }
    }
    Ok(Arc::new(catalog))
}

pub fn full_table_name(
    schema_name: impl Into<String>,
    table_name: impl Into<String>,
) -> String {
    format!("{}.{}", schema_name.into(), table_name.into())
}

fn infer_full_table_name(table_name: impl Into<String>) -> String {
    let name = table_name.into();
    if name.split('.').count() == 2 {
        name
    } else {
        full_table_name(APP_DB, name)
    }
}
