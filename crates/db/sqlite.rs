use crate::{
    Bundle, DeploymentId, DeploymentStatus, DeploymentType, Migration,
};
use anyhow::Result;
use rusqlite::types::{
    FromSql, FromSqlError, FromSqlResult, ToSqlOutput, Value, ValueRef,
};
use rusqlite::{params, ToSql};
use std::path::PathBuf;

impl FromSql for DeploymentType {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        match value.as_i64()? {
            0 => Ok(DeploymentType::Schema),
            1 => Ok(DeploymentType::Functions),
            _ => Err(FromSqlError::InvalidType),
        }
    }
}

impl ToSql for DeploymentType {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        match self {
            DeploymentType::Schema => Ok(ToSqlOutput::Owned(Value::Integer(0))),
            DeploymentType::Functions => {
                Ok(ToSqlOutput::Owned(Value::Integer(1)))
            }
        }
    }
}

impl FromSql for DeploymentStatus {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        match value.as_i64()? {
            0 => Ok(DeploymentStatus::Doing),
            1 => Ok(DeploymentStatus::Done),
            2 => Ok(DeploymentStatus::Failed),
            _ => Err(FromSqlError::InvalidType),
        }
    }
}

impl ToSql for DeploymentStatus {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        match self {
            DeploymentStatus::Doing => {
                Ok(ToSqlOutput::Owned(Value::Integer(0)))
            }
            DeploymentStatus::Done => Ok(ToSqlOutput::Owned(Value::Integer(1))),
            DeploymentStatus::Failed => {
                Ok(ToSqlOutput::Owned(Value::Integer(2)))
            }
        }
    }
}

pub async fn deploy_schema(
    project_id: &str,
    migrations: Vec<Migration>,
) -> Result<DeploymentId> {
    let sqlite_file = project_sqlite_file(project_id);
    let conn = rusqlite::Connection::open(sqlite_file.as_path())?;
    conn.execute(
        "INSERT INTO deployments (type, status) VALUES (?, ?)",
        params![DeploymentType::Schema, DeploymentStatus::Doing],
    )?;
    let deployment_id = conn.last_insert_rowid();
    for m in migrations.iter() {
        conn.execute(
            "INSERT INTO db_migrations (file_name, sql, applied, deployment_id) VALUES (?, ?, ?, ?)",
            params![&m.file_name, &m.sql, &DeploymentStatus::Doing, &deployment_id],
        )?;
    }

    for m in migrations.iter() {
        conn.execute_batch(m.sql.as_str())?;
        conn.execute(
            "UPDATE db_migrations SET applied = 1 WHERE file_name = ?",
            params![m.file_name],
        )?;
    }
    conn.execute(
        "UPDATE deployments SET status = ? WHERE id = ?",
        params![DeploymentStatus::Done, deployment_id],
    )?;
    Ok(deployment_id as u64)
}

pub async fn deploy_functions(
    project_id: &str,
    bundles: &Vec<Bundle>,
) -> Result<DeploymentId> {
    let sqlite_file = project_sqlite_file(project_id);
    let conn = rusqlite::Connection::open(sqlite_file.as_path())?;
    conn.execute(
        "INSERT INTO deployments (type, status) VALUES (?, ?)",
        params![DeploymentType::Functions, DeploymentStatus::Doing],
    )?;
    let deployment_id = conn.last_insert_rowid();
    for bundle in bundles.iter() {
        conn.execute("INSERT INTO js_bundles (path, code, deployment_id) VALUES (?, ?, ?)", params![bundle.path, bundle.code, deployment_id])?;
    }
    conn.execute(
        "UPDATE deployments SET status = ? WHERE id = ?",
        params![DeploymentStatus::Done, deployment_id],
    )?;
    Ok(deployment_id as u64)

    // store bundle in project's directory
    // let bundle_dir = project_bundle_dir(
    //     server_state.projects_dir.as_path(),
    //     project_id.as_str(),
    // );
    // for bundle in bundles.iter() {
    //     let bundle_file_path = PathBuf::from(bundle.path.as_str());
    //     if let Some(parent) = bundle_file_path.parent() {
    //         let mut parent_dir = bundle_dir.join(parent);
    //         std::fs::create_dir_all(parent_dir.as_path())?;
    //     }
    //     let file_path = bundle_dir.join(bundle.path.as_str());
    //     let mut file = File::create(file_path.as_path()).await?;
    //     file.write_all(bundle.code.as_bytes()).await?;
    // }
}

fn project_sqlite_file(project_id: &str) -> PathBuf {
    // todo: per project sqlite file!
    PathBuf::from("darx.sqlite")
}
