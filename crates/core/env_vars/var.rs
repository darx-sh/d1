use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum VarKind {
  Env,
  Deploy,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum SystemKey {
  Test,
  Invalid,
}

impl From<String> for SystemKey {
  fn from(s: String) -> Self {
    match s.as_str() {
      "test_key" => SystemKey::Test,
      _ => SystemKey::Invalid,
    }
  }
}

impl VarKind {
  pub(super) fn tbl_col(&self) -> (&str, &str, bool) {
    match self {
      VarKind::Env => ("env_vars", "env_id", true),
      VarKind::Deploy => ("deploy_vars", "deploy_id", false),
    }
  }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Var {
  key: String,
  val: String,
}

impl Var {
  pub fn new<T: Into<String>>(key: T, val: T) -> Var {
    Var {
      key: key.into(),
      val: val.into(),
    }
  }
  pub fn key(&self) -> &str {
    return &self.key;
  }
  pub fn val(&self) -> &str {
    return &self.val;
  }
}
