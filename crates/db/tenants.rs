use crate::TenantConnPool;
use anyhow::Context;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use dashmap::DashMap;
use futures_util::TryStreamExt;
use once_cell::sync::Lazy;
use serde::ser::SerializeMap;
use serde::{Serialize, Serializer};
use serde_json::Value;
use sqlx::mysql::MySqlConnectOptions;
use sqlx::mysql::MySqlRow;
use sqlx::MySqlPool;
use sqlx::{Column, Either, Row, TypeInfo};
use std::any::Any;
use std::ops::{Deref, DerefMut};

pub struct TenantDBInfo {
  pub host: String,
  pub port: u16,
  pub user: String,
  pub password: String,
  pub database: String,
}

pub fn test_tenant_db_info(env_id: &str) -> TenantDBInfo {
  TenantDBInfo {
    host: "localhost".to_string(),
    port: 3306,
    user: env_id.to_string(),
    password: env_id.to_string(),
    database: format!("dx_{}", env_id),
  }
}

static GLOBAL_POOL: Lazy<DashMap<String, MySqlPool>> = Lazy::new(DashMap::new);

static GLOBAL_DB_INFO: Lazy<DashMap<String, TenantDBInfo>> =
  Lazy::new(DashMap::new);

pub async fn get_tenant_pool(env_id: &str) -> Result<Box<dyn TenantConnPool>> {
  if let Some(pool) = GLOBAL_POOL.get(env_id) {
    Ok(Box::new(MySqlTenantConnection(pool.value().clone())))
  } else {
    let db_info = GLOBAL_DB_INFO
      .get(env_id)
      .ok_or_else(|| anyhow!("db info not found"))?;

    // todo: user per env lock to avoid duplicate connection
    let mysql_conn_options = MySqlConnectOptions::new()
      .host(&db_info.host)
      .port(db_info.port)
      .username(&db_info.user)
      .password(&db_info.password)
      .database(&db_info.database);
    let new_pool = MySqlPool::connect_with(mysql_conn_options).await?;
    let pool = GLOBAL_POOL
      .entry(env_id.to_string())
      .or_insert_with(|| new_pool.clone());
    Ok(Box::new(MySqlTenantConnection(pool.value().clone())))
  }
}

pub fn add_tenant_db_info(env_id: &str, db_info: TenantDBInfo) {
  GLOBAL_DB_INFO.insert(env_id.to_string(), db_info);
}

pub struct MySqlTenantConnection(MySqlPool);

impl Deref for MySqlTenantConnection {
  type Target = MySqlPool;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl DerefMut for MySqlTenantConnection {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

#[async_trait]
impl TenantConnPool for MySqlTenantConnection {
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
