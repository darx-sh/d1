use anyhow::{anyhow, Error, Result};
use async_trait::async_trait;
use darx_utils::test_mysql_db_url;
use mysql_async;
use mysql_async::{Conn, params};
use mysql_async::prelude::{Query, Queryable, WithParams};
use serde::ser::{Serialize, SerializeMap, Serializer};
use std::cell::RefCell;
use std::rc::Rc;

use crate::{Connection, ConnectionPool};

// impl From<DeploymentType> for mysql_async::Value {
//     fn from(t: DeploymentType) -> Self {
//         match t {
//             DeploymentType::Schema => mysql_async::Value::Int(0),
//             DeploymentType::Functions => mysql_async::Value::Int(1),
//         }
//     }
// }
//
// impl From<DeploymentStatus> for mysql_async::Value {
//     fn from(t: DeploymentStatus) -> Self {
//         match t {
//             DeploymentStatus::Doing => mysql_async::Value::Int(0),
//             DeploymentStatus::Done => mysql_async::Value::Int(1),
//             DeploymentStatus::Failed => mysql_async::Value::Int(2),
//         }
//     }
// }
//
// impl From<DBMigrationStatus> for mysql_async::Value {
//     fn from(value: DBMigrationStatus) -> Self {
//         match value {
//             DBMigrationStatus::Doing => mysql_async::Value::Int(0),
//             DBMigrationStatus::Done => mysql_async::Value::Int(1),
//             DBMigrationStatus::Failed => mysql_async::Value::Int(2),
//         }
//     }
// }

// pub async fn deploy_schema(
//     project_id: &str,
//     migrations: Vec<Migration>,
// ) -> Result<DeploymentId> {
//     let mut conn = raw_conn(project_id).await?;
//     let res = r"INSERT INTO deployments (project_id, type, status) VALUES (:project_id, :type, :status)"
//         .with(params! {
//             "project_id" => project_id,
//             "type" => DeploymentType::Schema,
//             "status" => DeploymentStatus::Doing,
//         })
//         .run(&mut conn)
//         .await?;
//
//     let deployment_id = res.last_insert_id().unwrap();
//
//     for m in migrations.iter() {
//         r"INSERT INTO db_migrations (file_name, sql, status, deployment_id) VALUES (:file_name, :sql, :status, :deployment_id)".with(params!{
//             "file_name" => &m.file_name,
//             "sql" => &m.sql,
//             "status" => DBMigrationStatus::Doing,
//             "deployment_id" => &deployment_id,
//         }).run(&mut conn).await?;
//     }
//
//     for m in migrations.iter() {
//         match m.sql.as_str().run(&mut conn).await {
//             Ok(_) => {
//                 r"UPDATE db_migrations SET status = :status WHERE file_name = :file_name"
//                     .with(params! {
//                 "status" => DBMigrationStatus::Done,
//                 "file_name" => &m.file_name,
//                     })
//                     .run(&mut conn)
//                     .await?;
//             }
//             Err(e) => {
//                 r"UPDATE db_migrations SET status = :status, error = :error WHERE file_name = :file_name"
//                     .with(params! {
//                 "status" => DBMigrationStatus::Failed,
//                         "error" => e.to_string(),
//                 "file_name" => &m.file_name,
//                     })
//                     .run(&mut conn)
//                     .await?;
//
//                 r"UPDATE deployments SET status = :status WHERE id = :deployment_id"
//                     .with(params! {
//             "status" => DeploymentStatus::Failed,
//             "deployment_id" => &deployment_id,
//                     })
//                     .run(&mut conn)
//                     .await?;
//                 return Err(anyhow!("failed to run migration {}", m.file_name));
//             }
//         }
//     }
//
//     r"UPDATE deployments SET status = :status WHERE id = :deployment_id"
//         .with(params! {
//             "status" => DeploymentStatus::Done,
//             "deployment_id" => &deployment_id,
//         })
//         .run(&mut conn)
//         .await?;
//     Ok(deployment_id)
// }

// pub async fn deploy_functions(
//     project_id: &str,
//     bundles: &Vec<Bundle>,
//     meta: &serde_json::Value,
// ) -> Result<DeploymentId> {
//     let mut conn = raw_conn(project_id).await?;
//     let res = r"INSERT INTO deployments (project_id, type, status) VALUES (:project_id, :type, :status)"
//         .with(params! {
//             "project_id" => project_id,
//             "type" => DeploymentType::Functions,
//             "status" => DeploymentStatus::Doing,
//         })
//         .run(&mut conn)
//         .await?;
//     let deployment_id = res.last_insert_id().unwrap();
//
//     match insert_code(&mut conn, deployment_id, bundles, meta).await {
//         Ok(_) => {
//             let mut txn = conn
//                 .start_transaction(mysql_async::TxOpts::default())
//                 .await?;
//
//             r"UPDATE deployments SET status = :status WHERE id = :deployment_id"
//                 .with(params! {
//                     "status" => DeploymentStatus::Done,
//                     "deployment_id" => &deployment_id,
//                 })
//                 .run(&mut txn)
//                 .await?;
//             r"REPLACE INTO current_deployments (project_id, deployment_id) VALUES (:project_id, :deployment_id)"
//                 .with(params! {
//                     "project_id" => project_id,
//                     "deployment_id" => &deployment_id,
//                 })
//                 .run(&mut txn)
//                 .await?;
//             txn.commit().await?;
//         }
//         Err(e) => {
//             r"UPDATE deployments SET status = :status WHERE id = :deployment_id"
//                 .with(params! {
//                     "status" => DeploymentStatus::Failed,
//                     "deployment_id" => &deployment_id,
//                 })
//                 .run(&mut conn)
//                 .await?;
//             return Err(e);
//         }
//     }
//     Ok(deployment_id)
// }

// pub async fn load_bundles_from_db(
//     project_id: &str,
//     deployment_id: &str,
// ) -> Result<(Vec<Bundle>, serde_json::Value)> {
//     let mut conn = raw_conn(project_id).await?;
//     let bundles = r"SELECT path, code FROM js_bundles WHERE deployment_id = :deployment_id"
//         .with(params! {
//             "deployment_id" => deployment_id,
//         }).map(&mut conn, |(path, code)| Bundle{path, code}).await?;
//
//     let meta: Option<String> =
//         r"SELECT meta FROM js_bundle_metas WHERE deployment_id = :deployment_id"
//             .with(params! {
//                 "deployment_id" => deployment_id,
//             })
//             .first(&mut conn)
//             .await?;
//     let meta = meta.ok_or_else(|| {
//         anyhow!("no bundle meta found for deployment {}", deployment_id)
//     })?;
//
//     Ok((bundles, serde_json::from_str(&meta)?))
// }
//
// async fn insert_code(
//     conn: &mut Conn,
//     deployment_id: DeploymentId,
//     bundles: &Vec<Bundle>,
//     meta: &serde_json::Value,
// ) -> Result<()> {
//     for bundle in bundles.iter() {
//         r"INSERT INTO js_bundles (path, code, deployment_id) VALUES (:path, :code, :deployment_id)".with(params!{
//             "path" => &bundle.path,
//             "code" => &bundle.code,
//             "deployment_id" => &deployment_id,
//         }).run(&mut *conn).await?;
//     }
//
//     r"INSERT INTO js_bundle_metas (deployment_id, meta) VALUES (:deployment_id, :meta)".with(params!{
//         "deployment_id" => &deployment_id,
//         "content" => meta.to_string(),
//     }).run(conn).await?;
//     Ok(())
// }

async fn raw_conn(project_id: &str) -> mysql_async::Result<Conn> {
    // todo: per project connection pool
    mysql_async::Conn::from_url(test_mysql_db_url()).await
}

pub struct MySqlPool {
    pool: mysql_async::Pool,
}

impl MySqlPool {
    pub fn new(url: &str) -> Self {
        MySqlPool {
            pool: mysql_async::Pool::new(url),
        }
    }
}

#[async_trait]
impl ConnectionPool for MySqlPool {
    async fn get_conn(&self) -> Result<Rc<RefCell<dyn Connection>>> {
        let conn = self.pool.get_conn().await?;
        Ok(Rc::new(RefCell::new(MySqlConn { conn })))
    }
}

pub struct MySqlConn {
    conn: mysql_async::Conn,
}

#[async_trait]
impl Connection for MySqlConn {
    async fn execute(
        &mut self,
        query: &str,
        params: Vec<serde_json::Value>,
    ) -> Result<serde_json::Value> {
        let params: Vec<XDatum> =
            params.into_iter().map(|v| v.into()).collect();

        let mut query_result = self.conn.exec_iter(query, params).await?;
        let r = if query_result.is_empty() {
            MySqlQueryResult::Empty {
                affected_rows: query_result.affected_rows(),
                last_insert_id: query_result.last_insert_id(),
            }
        } else {
            let xrows = query_result.map(|r| r.into()).await?;
            MySqlQueryResult::Rows(xrows)
        };
        Ok(serde_json::to_value(r)?)
    }
}

struct XDatum(mysql_async::Value);

impl From<serde_json::Value> for XDatum {
    fn from(v: serde_json::Value) -> Self {
        match v {
            serde_json::Value::String(s) => {
                return XDatum(mysql_async::Value::Bytes(s.into_bytes()));
            }
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    return XDatum(mysql_async::Value::Int(i));
                }
                if let Some(u) = n.as_u64() {
                    return XDatum(mysql_async::Value::UInt(u));
                }
                if let Some(f) = n.as_f64() {
                    return XDatum(mysql_async::Value::Double(f));
                }
                unimplemented!("Number type not supported");
            }
            _ => {
                unimplemented!()
            }
        }
    }
}

impl From<XDatum> for mysql_async::Value {
    fn from(v: XDatum) -> Self {
        v.0
    }
}

impl From<&mysql_async::Value> for XDatum {
    fn from(v: &mysql_async::Value) -> Self {
        XDatum(v.clone())
    }
}

struct XValue {
    pub datum: XDatum,
    pub ty: mysql_async::consts::ColumnType,
}

impl Serialize for XValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match &self.datum.0 {
            mysql_async::Value::Bytes(bytes) if self.ty.is_character_type() => {
                serializer
                    .serialize_str(&String::from_utf8(bytes.clone()).unwrap())
            }
            mysql_async::Value::Int(i) => serializer.serialize_i64(*i),
            mysql_async::Value::UInt(i) => serializer.serialize_u64(*i),
            mysql_async::Value::Float(f) => serializer.serialize_f32(*f),
            mysql_async::Value::Double(f) => serializer.serialize_f64(*f),
            _ => {
                unimplemented!()
            }
        }
    }
}

struct XColumn {
    pub name: String,
    pub value: XValue,
}

struct XRow(pub Vec<XColumn>);

impl Serialize for XRow {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for column in &self.0 {
            map.serialize_entry(&column.name, &column.value)?;
        }
        map.end()
    }
}

impl From<mysql_async::Row> for XRow {
    fn from(row: mysql_async::Row) -> Self {
        let mut xrow = XRow(vec![]);
        let columns = row.columns_ref();
        for i in 0..columns.len() {
            let cname = columns[i].name_str();
            let ty = columns[i].column_type();
            let datum: XDatum = row.as_ref(i).unwrap().into();
            let column = XColumn {
                name: cname.to_string(),
                value: XValue { datum, ty },
            };
            xrow.0.push(column);
        }
        xrow
    }
}

enum MySqlQueryResult {
    Rows(Vec<XRow>),
    Empty {
        affected_rows: u64,
        last_insert_id: Option<u64>,
    },
}

impl Serialize for MySqlQueryResult {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            MySqlQueryResult::Rows(rows) => rows.serialize(serializer),
            MySqlQueryResult::Empty {
                affected_rows,
                last_insert_id,
            } => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("affected_rows", affected_rows)?;
                map.serialize_entry("last_insert_id", last_insert_id)?;
                map.end()
            }
        }
    }
}
