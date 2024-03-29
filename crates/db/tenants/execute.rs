use crate::tenants::pool::MySqlTenantPool;
use crate::tenants::TenantConnPool;
use anyhow::Context;
use anyhow::Result;
use async_trait::async_trait;
use futures_util::TryStreamExt;
use serde::ser::SerializeMap;
use serde::{Serialize, Serializer};
use serde_json::Value;
use sqlx::mysql::MySqlRow;
use sqlx::{Column, Either, Row, TypeInfo};
use std::any::Any;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

#[async_trait]
impl TenantConnPool for MySqlTenantPool {
  async fn js_execute(&self, sql: &str, params: Vec<Value>) -> Result<Value> {
    let mut query = sqlx::query(sql);
    for p in params.iter() {
      match p {
        // we Option<String> here because sqlx::query() doesn't have native Null type.
        Value::Null => query = query.bind::<Option<String>>(None),
        Value::Bool(v) => query = query.bind::<bool>(*v),
        Value::Number(v) => {
          if v.is_i64() {
            query = query.bind::<i64>(v.as_i64().unwrap());
          } else if v.is_u64() {
            query = query.bind::<u64>(v.as_u64().unwrap());
          } else if v.is_f64() {
            query = query.bind::<f64>(v.as_f64().unwrap());
          } else {
            unimplemented!()
          }
        }
        Value::String(v) => query = query.bind::<String>(v.to_string()),
        Value::Array(v) => {
          let mut arr = sqlx::types::Json::<Vec<Value>>::default();
          arr.0 = v.clone();
          query = query.bind::<sqlx::types::Json<Vec<Value>>>(arr);
        }
        Value::Object(v) => {
          let mut obj =
            sqlx::types::Json::<serde_json::Map<String, Value>>::default();
          obj.0 = v.clone();
          query = query
            .bind::<sqlx::types::Json<serde_json::Map<String, Value>>>(obj);
        }
      }
    }

    let mut result_set = ResultSet::default();
    let pool = &self.0;
    let mut stream = query.fetch_many(pool);
    while let Some(r) = stream
      .try_next()
      .await
      .with_context(|| "Failed to get result from query")?
    {
      match r {
        Either::Left(r) => {
          result_set.rowsAffected = r.rows_affected();
          result_set.lastInsertId = Some(r.last_insert_id());
        }
        Either::Right(r) => {
          let row = XRow(r);
          result_set.rows.push(row);
        }
      }
    }
    Ok(serde_json::to_value(result_set)?)
  }

  fn as_any(&self) -> &dyn Any {
    self
  }
}

struct XRow(MySqlRow);

impl Serialize for XRow {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    let columns = self.0.columns();
    let mut map = serializer.serialize_map(Some(columns.len()))?;
    for (idx, column) in columns.iter().enumerate() {
      let name = column.name();
      let type_info = column.type_info();
      let type_name = type_info.name();

      // There is an issue whe use try_get(name): https://github.com/launchbadge/sqlx/issues/2206
      // which leads to "ColumnNotFound", so we use try_get(idx) instead.
      match type_name {
        "BOOLEAN" => {
          let v: Option<bool> = self.0.try_get(idx).unwrap();
          map.serialize_entry(name, &v)?;
        }
        "TINYINT UNSIGNED" | "SMALLINT UNSIGNED" | "INT UNSIGNED"
        | "MEDIUMINT UNSIGNED" | "BIGINT UNSIGNED" => {
          let v: Option<u64> = self.0.try_get(idx).unwrap();
          map.serialize_entry(name, &v)?;
        }
        "TINYINT" | "SMALLINT" | "INT" | "MEDIUMINT" | "BIGINT" => {
          let v: Option<i64> = self.0.try_get(idx).unwrap();
          map.serialize_entry(name, &v)?;
        }
        "FLOAT" => {
          let v: Option<f32> = self.0.try_get(idx).unwrap();
          map.serialize_entry(name, &v)?;
        }
        "DOUBLE" => {
          let v: Option<f64> = self.0.try_get(idx).unwrap();
          map.serialize_entry(name, &v)?;
        }
        "DECIMAL" => {
          // todo: why?
          let v: Option<sqlx::types::BigDecimal> = self.0.try_get(idx).unwrap();
          let v = v.map(|v| format!("{}", v));
          map.serialize_entry(name, &v)?;
        }
        "NULL" => {
          let v: Option<String> = self.0.try_get(idx).unwrap();
          map.serialize_entry(name, &v)?;
        }
        "TIMESTAMP" | "DATETIME" => {
          let v: Option<OffsetDateTime> = self.0.try_get(idx).unwrap();
          if let Some(v) = v {
            map.serialize_entry(name, &v.format(&Rfc3339).unwrap())?;
          } else {
            map.serialize_entry(name, &None::<String>)?;
          }
        }
        "DATE" => {
          let v: Option<time::Date> = self.0.try_get(idx).unwrap();
          let v = v.map(|v| format!("{}", v));
          map.serialize_entry(name, &v)?;
        }
        "TIME" => {
          let v: Option<time::Time> = self.0.try_get(idx).unwrap();
          let v = v.map(|v| format!("{}", v));
          map.serialize_entry(name, &v)?;
        }
        "YEAR" => {
          let v: Option<String> = self.0.try_get(idx).unwrap();
          map.serialize_entry(name, &v)?;
        }
        "BIT" => {
          let v: Option<Vec<u8>> = self.0.try_get(idx).unwrap();
          map.serialize_entry(name, &v)?;
        }
        "ENUM" => {
          let v: Option<String> = self.0.try_get(idx).unwrap();
          map.serialize_entry(name, &v)?;
        }
        "SET" => {
          let v: Option<String> = self.0.try_get(idx).unwrap();
          map.serialize_entry(name, &v)?;
        }
        "CHAR" | "VARCHAR" | "TEXT" | "LONGTEXT" => {
          let v: Option<String> = self.0.try_get(idx).unwrap();
          map.serialize_entry(name, &v)?;
        }
        other => unimplemented!("{}", other),
      }
    }
    map.end()
  }
}

#[allow(non_snake_case)]
#[derive(Serialize, Default)]
struct ResultSet {
  rows: Vec<XRow>,
  rowsAffected: u64,
  lastInsertId: Option<u64>,
}
