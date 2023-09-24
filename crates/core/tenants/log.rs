use anyhow::{Context, Result};
use sqlx::MySqlExecutor;
use time::{Duration, OffsetDateTime, PrimitiveDateTime};
use tracing::info;

#[derive(sqlx::FromRow, PartialEq, Debug)]
pub struct LogPO {
  pub id: i64,
  pub env_id: String,
  pub deploy_seq: i64,
  pub time: PrimitiveDateTime,
  pub level: i32,
  pub func: String,
  pub message: String,
}

impl LogPO {
  pub async fn find<'c>(
    exe: impl MySqlExecutor<'c>,
    env: &str,
    seq: i64,
    start_time: OffsetDateTime,
    end_time: OffsetDateTime,
    func: Option<&str>,
  ) -> Result<Vec<LogPO>> {
    assert!(!env.is_empty());

    if func.is_none() {
      sqlx::query_as!(
      LogPO,
      "select id, env_id, deploy_seq, time, level, func, message from deploy_log where env_id = ? and deploy_seq = ? and time between ? and ?",
      env, seq, start_time, end_time
    ).fetch_all(exe).await.with_context(|| {
        format!(
          "error find log. env {}, seq {}, start {}, end {}, func {:?}",
          env, seq, start_time, end_time, func
        )
      })
    } else {
      sqlx::query_as!(
      LogPO,
      "select id, env_id, deploy_seq, time, level, func, message from deploy_log where env_id = ? and deploy_seq = ? and time between ? and ? and func like ?",
      env, seq, start_time, end_time, func
    ).fetch_all(exe).await.with_context(|| {
        format!(
          "error find log. env {}, seq {}, start {}, end {}, func {:?}",
          env, seq, start_time, end_time, func
        )
      })
    }
  }
  pub async fn find_recent<'c>(
    exe: impl MySqlExecutor<'c>,
    env: &str,
    seq: i64,
    duration: Duration,
    func: Option<&str>,
  ) -> Result<Vec<LogPO>> {
    assert!(!env.is_empty());

    let end_time = OffsetDateTime::now_utc();
    let start_time = end_time - duration;
    LogPO::find(exe, env, seq, start_time, end_time, func).await
  }

  pub async fn save<'c>(
    exe: impl MySqlExecutor<'c>,
    logs: &Vec<LogPO>,
  ) -> Result<u64> {
    let values = logs
      .iter()
      .map(|e| {
        format!(
          "('{}', '{}', '{}', '{}', '{}', '{}')",
          &e.env_id, e.deploy_seq, e.time, e.level, &e.func, &e.message,
        )
      })
      .collect::<Vec<String>>()
      .join(", ");
    let sql = format!(
      "insert into `deploy_log` (`env_id`, `deploy_seq`, `time`, `level`, `func`, `message`) values {}", values
    );
    let r = exe
      .execute(sql.as_str())
      .await
      .map(|r| r.rows_affected())
      .context("error save logs");

    if r.is_ok() {
      info!("inserted {} logs", r.as_ref().unwrap());
    }

    r
  }
}

#[cfg(test)]
mod tests {
  use std::env;

  use anyhow::Context;
  use sqlx::MySqlPool;
  use tracing::warn;

  use super::*;

  #[tokio::test]
  async fn test_basic() -> Result<()> {
    let has_db = env::var("DATABASE_URL").is_ok();

    if !has_db {
      warn!("skip don't has DATABASE_URL");
      return Ok(());
    }

    let db = MySqlPool::connect(
      env::var("DATABASE_URL")
        .expect("DATABASE_URL should be configured")
        .as_str(),
    )
    .await
    .context("Failed to connect database")?;

    let mut txn = db.begin().await.unwrap();
    let env = "env".to_string();
    let seq = 1;

    let logs = vec![
      LogPO {
        id: 0,
        env_id: env.clone(),
        deploy_seq: seq,
        time: Default::default(),
        level: 0,
        func: "fffaaa".to_string(),
        message: "aaa".to_string(),
      },
      LogPO {
        id: 0,
        env_id: env.to_string(),
        deploy_seq: seq,
        time: Default::default(),
        level: 0,
        func: "fffbbb".to_string(),
        message: "bbb".to_string(),
      },
    ];

    let inserted = LogPO::save(&mut *txn, &logs).await.unwrap();
    assert_eq!(inserted, logs.len() as u64);
    let found =
      LogPO::find_recent(&mut *txn, &env, seq, Duration::hours(1), None)
        .await
        .unwrap();
    assert_eq!(logs, found);

    let found =
      LogPO::find_recent(&mut *txn, &env, seq, Duration::hours(1), Some("fff"))
        .await
        .unwrap();
    assert_eq!(logs, found);

    let found = LogPO::find_recent(
      &mut *txn,
      &env,
      seq,
      Duration::hours(1),
      Some("fffa%"),
    )
    .await
    .unwrap();
    assert_eq!(&logs[0..1], found);

    let found = LogPO::find_recent(
      &mut *txn,
      &env,
      seq,
      Duration::hours(1),
      Some("fffb%"),
    )
    .await
    .unwrap();
    assert_eq!(&logs[1..], found);

    let found = LogPO::find_recent(
      &mut *txn,
      &env,
      seq,
      Duration::hours(1),
      Some("fffc%"),
    )
    .await
    .unwrap();
    assert!(found.is_empty());

    txn.rollback().await?;
    Ok(())
  }
}
