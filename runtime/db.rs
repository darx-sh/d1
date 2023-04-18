use deno_core::error::AnyError;
use deno_core::{op, ResourceId};
use deno_core::{OpState, Resource};
use serde::{Deserialize, Serialize};
use serde_json;
use sqlx::database::HasValueRef;
use sqlx::error::BoxDynError;
use sqlx::mysql::{MySqlArguments, MySqlTypeInfo};
use sqlx::{Column, Decode, MySql, MySqlPool, Row, Type, TypeInfo, ValueRef};
use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

deno_core::extension!(
    darx_db,
    deps = [darx_bootstrap],
    ops = [op_db_fetch_all],
    esm = ["js/01_db.js"]
);

#[op]
pub async fn op_db_fetch_all(
    state: Rc<RefCell<OpState>>,
    query_str: String,
    params: Vec<serde_json::Value>,
) -> Result<Vec<DarxRow>, AnyError> {
    let pool = state.borrow().borrow::<MySqlPool>().clone();

    let mut query = sqlx::query(&query_str);
    for param in params {
        query = query.bind(param);
    }
    let rows = query.fetch_all(&pool).await?;
    let mut drows = vec![];
    rows.iter().for_each(|row| {
        let num_columns = row.len();
        let mut drow = DarxRow(HashMap::new());
        let columns = row.columns();
        for i in 0..num_columns {
            let v: DarxColumnValue = row.try_get(i).unwrap();
            drow.0.insert(columns[i].name().to_string(), v);
        }
        drows.push(drow);
    });
    Ok(drows)
}

#[derive(Serialize, Deserialize)]
struct DarxColumnValue(serde_json::Value);

#[derive(Serialize, Deserialize)]
struct DarxRow(HashMap<String, DarxColumnValue>);

impl Type<MySql> for DarxColumnValue {
    fn type_info() -> MySqlTypeInfo {
        // it is not used in decode.
        todo!()
    }

    fn compatible(_ty: &MySqlTypeInfo) -> bool {
        true
    }
}

impl Decode<'_, MySql> for DarxColumnValue {
    fn decode(
        value: <MySql as HasValueRef<'_>>::ValueRef,
    ) -> Result<Self, BoxDynError> {
        let type_info = value.type_info().to_mut().clone();
        // todo: these type conversion are not tested
        match type_info.name() {
            "BOOLEAN" => {
                let v = serde_json::Value::Bool(
                    <bool as Decode<MySql>>::decode(value)?,
                );
                Ok(Self(v))
            }
            // uint
            "TINYINT UNSIGNED" | "SMALLINT UNSIGNED" | "INT UNSIGNED"
            | "MEDIUMINT UNSIGNED" | "BIGINT UNSIGNED" => {
                let v = serde_json::Value::Number(
                    <u64 as Decode<MySql>>::decode(value)?.into(),
                );
                Ok(Self(v))
            }
            // int
            "TINYINT" | "SMALLINT" | "INT" | "MEDIUMINT" | "BIGINT" => {
                let v = serde_json::Value::Number(
                    <i64 as Decode<MySql>>::decode(value)?.into(),
                );
                Ok(Self(v))
            }
            // float
            "FLOAT" | "DOUBLE" => {
                let number =
                    serde_json::Number::from_f64(
                        <f64 as Decode<MySql>>::decode(value)?,
                    )
                    .unwrap();
                Ok(Self(serde_json::Value::Number(number)))
            }
            "NULL" => {
                let v = serde_json::Value::Null;
                Ok(Self(v))
            }
            "TIMESTAMP" | "DATE" | "TIME" | "DATETIME" | "YEAR" => {
                let v = serde_json::Value::String(
                    <&str as Decode<MySql>>::decode(value)?.into(),
                );
                Ok(Self(v))
            }
            "BIT" => {
                let v = serde_json::Value::String(
                    <&str as Decode<MySql>>::decode(value)?.into(),
                );
                Ok(Self(v))
            }
            "ENUM" => {
                let v = serde_json::Value::String(
                    <&str as Decode<MySql>>::decode(value)?.into(),
                );
                Ok(Self(v))
            }
            "SET" => {
                let v = serde_json::Value::String(
                    <&str as Decode<MySql>>::decode(value)?.into(),
                );
                Ok(Self(v))
            }
            "DECIMAL" => {
                let v = serde_json::Value::String(
                    <&str as Decode<MySql>>::decode(value)?.into(),
                );
                Ok(Self(v))
            }
            "GEOMETRY" => {
                let v = serde_json::Value::String(
                    <&str as Decode<MySql>>::decode(value)?.into(),
                );
                Ok(Self(v))
            }
            "JSON" => {
                let s = <&str as Decode<MySql>>::decode(value)?;
                let v: serde_json::Value = serde_json::from_str(s)?;
                Ok(Self(v))
            }
            "BINARY" => {
                let v = serde_json::Value::String(
                    <&str as Decode<MySql>>::decode(value)?.into(),
                );
                Ok(Self(v))
            }
            "CHAR" | "VARCHAR" | "TINYTEXT" | "TEXT" | "MEDIUMTEXT"
            | "LONGTEXT" => {
                let v = serde_json::Value::String(
                    <&str as Decode<MySql>>::decode(value)?.into(),
                );
                Ok(Self(v))
            }
            _ => todo!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{create_db_pool, DarxRuntime};
    use deno_core::anyhow::Result;
    use deno_core::futures::TryStreamExt;
    use serde_json;
    use sqlx::{Column, MySqlPool, Row};
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_db_query() -> Result<()> {
        let pool = create_db_pool().await;
        sqlx::query("CREATE TABLE IF NOT EXISTS test (id INT NOT NULL AUTO_INCREMENT, name VARCHAR(255) NOT NULL, PRIMARY KEY (id))")
            .execute(&pool)
            .await?;
        let tenant_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("examples/tenants/7ce52fdc14b16017");
        let mut darx_runtime = DarxRuntime::new(pool, tenant_path);
        darx_runtime.run("run_query.js").await?;
        Ok(())
    }
}
