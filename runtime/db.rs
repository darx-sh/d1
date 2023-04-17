use serde::{Deserialize, Serialize};
use serde_json;
use sqlx::database::HasValueRef;
use sqlx::error::BoxDynError;
use sqlx::mysql::MySqlTypeInfo;
use sqlx::{Decode, MySql, Type, TypeInfo, ValueRef};
use std::collections::HashMap;

// deno_core::extension!(darx_db, ops = [op_db_query], esm = ["js/db.js"],);

#[derive(Serialize, Deserialize)]
struct ColumnValue(serde_json::Value);

#[derive(Serialize, Deserialize)]
struct Row(HashMap<String, ColumnValue>);

impl Type<MySql> for ColumnValue {
    fn type_info() -> MySqlTypeInfo {
        // it is not used in decode.
        todo!()
    }

    fn compatible(_ty: &MySqlTypeInfo) -> bool {
        true
    }
}

impl Decode<'_, MySql> for ColumnValue {
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
    use deno_core::anyhow::Result;
    use deno_core::futures::TryStreamExt;
    use serde_json;
    use sqlx::{Column, MySqlPool, Row};

    #[tokio::test]
    async fn test_fetch() -> Result<()> {
        let pool =
            MySqlPool::connect("mysql://root:12345678@localhost:3306/test")
                .await?;

        sqlx::query("CREATE TABLE IF NOT EXISTS test (id INT NOT NULL AUTO_INCREMENT, name VARCHAR(255) NOT NULL, PRIMARY KEY (id))")
            .execute(&pool)
            .await?;

        let q = sqlx::query("SELECT * FROM test");

        let rows = q.fetch_all(&pool).await?;

        rows.iter().for_each(|row| {
            let num_columns = row.len();
            let mut my_row = Row(HashMap::new());
            let columns = row.columns();
            for i in 0..num_columns {
                let v: ColumnValue = row.try_get(i).unwrap();
                my_row.0.insert(columns[i].name().to_string(), v);
            }
            println!("my_row: {}", serde_json::to_string(&my_row).unwrap());
        });

        Ok(())
    }
}
