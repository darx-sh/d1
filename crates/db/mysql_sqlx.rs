use crate::{Connection, ConnectionPool};
use anyhow::Result;
use async_trait::async_trait;
use futures::StreamExt;
use futures_util::TryStreamExt;
use serde::ser::SerializeMap;
use serde::{Serialize, Serializer};
use serde_json::Value;
use sqlx::mysql::{MySqlRow, MySqlValue};
use sqlx::{Column, Either, Encode, MySql, Row, Type, TypeInfo};
use std::cell::RefCell;
use std::rc::Rc;

pub struct MySqlPool {
    pool: sqlx::mysql::MySqlPool,
}

impl MySqlPool {
    pub async fn new(url: &str) -> Result<Self> {
        Ok(Self {
            pool: sqlx::mysql::MySqlPool::connect(url).await?,
        })
    }
}

#[async_trait]
impl ConnectionPool for MySqlPool {
    async fn get_conn(&self) -> Result<Rc<RefCell<dyn Connection>>> {
        let conn = self.pool.acquire().await?;
        Ok(Rc::new(RefCell::new(MySqlConn { conn })))
    }
}

pub struct MySqlConn {
    conn: sqlx::pool::PoolConnection<MySql>,
}

#[async_trait]
impl Connection for MySqlConn {
    async fn execute(
        &mut self,
        query_str: &str,
        params: Vec<Value>,
    ) -> Result<Value> {
        // self.conn.
        let mut query = sqlx::query(query_str);
        for p in params.iter() {
            match p {
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
                    let mut obj = sqlx::types::Json::<
                        serde_json::Map<String, Value>,
                    >::default();
                    obj.0 = v.clone();
                    query = query.bind::<sqlx::types::Json<
                        serde_json::Map<String, Value>,
                    >>(obj);
                }
            }
        }
        // let a = query.fetch_many(&mut self.conn).map_ok(|e| match e {
        //     Either::Left(l) => todo!(),
        //     Either::right(r) => todo!(),
        // });
        // let mut final_result: MySqlFetchResult;
        // let mut rows = vec![];
        // for r in fetch_result.into_iter() {
        //     match r {
        //         Ok(Either::Left(r)) => {
        //             final_result = MySqlFetchResult::Empty {
        //                 affected_rows: r.affected_rows(),
        //                 last_insert_id: Some(r.last_insert_id()),
        //             };
        //             break;
        //         }
        //         Ok(Either::Right(r)) => {
        //             let row = XRow(r);
        //             rows.push(row);
        //         }
        //         Err(e) => return Err(e.into()),
        //     }
        // }
        todo!()
        // Ok(serde_json::to_value(fetch_result)?)
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

enum MySqlFetchResult {
    Row(Vec<XRow>),
    Empty {
        affected_rows: u64,
        last_insert_id: Option<u64>,
    },
}
