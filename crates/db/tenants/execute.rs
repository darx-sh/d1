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
    for column in columns {
      let name = column.name();
      let type_info = column.type_info();

      let type_name = type_info.name();
      match type_name {
        "INT" | "BIGINT" => {
          let v: Option<i64> = self.0.try_get(name).unwrap();
          map.serialize_entry(name, &v)?;
        }
        "VARCHAR" => {
          let v: Option<String> = self.0.try_get(name).unwrap();
          map.serialize_entry(name, &v)?;
        }
        _ => unimplemented!(),
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
