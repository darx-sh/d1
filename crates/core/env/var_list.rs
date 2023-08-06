use crate::env::var::{Var, VarKind};
use anyhow::{Context, Result};
use sqlx::{Executor, MySql, Row, Transaction};
use tracing::info;

#[derive(Debug, PartialEq)]
pub struct VarList {
  kind: VarKind,
  parent_id: String,
  vars: Vec<Var>,
}

impl VarList {
  pub fn env_to_deploy(mut self, deploy_id: &str) -> Self {
    assert_eq!(self.kind, VarKind::Env);
    self.kind = VarKind::Deploy;
    self.parent_id = deploy_id.to_string();
    self
  }

  pub async fn save<'c>(
    &self,
    txn: &mut Transaction<'c, MySql>,
  ) -> Result<u64> {
    if self.vars.is_empty() {
      return Ok(0);
    }

    let (tbl, parent) = self.kind.tbl_col();
    let values = self
      .vars
      .iter()
      .map(|v| {
        format!("('{}', '{}', '{}')", &self.parent_id, v.key(), &v.val())
      })
      .collect::<Vec<String>>()
      .join(", ");
    let sql = format!(
      "insert into `{}` (`{}`, `key`, `value`) values {}",
      tbl, parent, values
    );
    let r = txn
      .execute(sql.as_str())
      .await
      .map(|r| r.rows_affected())
      .context("error save var list");

    if r.is_ok() {
      info!("inserted {} {:?} var list", self.vars.len(), self.kind);
    }

    r
  }

  pub async fn delete<'c>(
    self,
    txn: &mut Transaction<'c, MySql>,
  ) -> Result<u64> {
    if self.vars.is_empty() {
      return Ok(0);
    }

    let (tbl, parent_col) = self.kind.tbl_col();
    let sql = format!(
      "delete from {} where {} = '{}'",
      tbl, parent_col, &self.parent_id
    );

    let r = txn
      .execute(sql.as_str())
      .await
      .map(|r| r.rows_affected())
      .context("error del var list");

    if r.is_ok() {
      info!("deleted {} {:?} var list", self.vars.len(), self.kind);
    }

    r
  }

  pub async fn find<'c>(
    txn: &mut Transaction<'c, MySql>,
    parent_id: &str,
    kind: VarKind,
  ) -> Result<VarList> {
    assert!(!parent_id.is_empty());

    let (tbl, parent_col) = kind.tbl_col();
    let sql = format!(
      "select `key`, `value` from {} where {} = '{}'",
      tbl, parent_col, parent_id
    );

    let mut ret = VarList {
      kind,
      parent_id: parent_id.to_string(),
      vars: vec![],
    };

    txn
      .fetch_all(sql.as_str())
      .await
      .with_context(|| {
        format!(
          "error list var list. parent {}, kind {:?}",
          parent_id, &kind
        )
      })?
      .iter()
      .for_each(|r| {
        ret.vars.push(Var::new(r.get("key"), r.get("value")));
      });

    Ok(ret)
  }
}

#[cfg(test)]
mod tests {
  use std::env;

  use anyhow::Context;
  use sqlx::MySqlPool;

  use super::*;

  #[tokio::test]
  async fn test_basic() -> Result<()> {
    let db = MySqlPool::connect(
      env::var("DATABASE_URL")
        .expect("DATABASE_URL should be configured")
        .as_str(),
    )
    .await
    .context("Failed to connect database")?;

    let parent_id = "test_parent";

    let mut list = VarList {
      kind: VarKind::Env,
      parent_id: parent_id.to_string(),
      vars: vec![
        Var::new("test_key1".to_string(), "test_val1".to_string()),
        Var::new("test_key2".to_string(), "test_val2".to_string()),
      ],
    };

    let mut txn = db.begin().await.unwrap();
    list.save(&mut txn).await.unwrap();
    let actual = VarList::find(&mut txn, parent_id, list.kind).await?;
    assert_eq!(list, actual);
    let deleted = actual.delete(&mut txn).await?;
    assert_eq!(list.vars.len() as u64, deleted);

    list.kind = VarKind::Deploy;
    list.save(&mut txn).await.unwrap();
    let actual = VarList::find(&mut txn, parent_id, list.kind).await?;
    assert_eq!(list, actual);
    let deleted = actual.delete(&mut txn).await?;
    assert_eq!(list.vars.len() as u64, deleted);
    txn.rollback().await?;
    Ok(())
  }
}
