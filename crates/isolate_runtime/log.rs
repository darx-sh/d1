use once_cell::sync::Lazy;
use regex::Regex;
use std::cell::RefCell;
use time::OffsetDateTime;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Level(pub i32);

// standard levels should keep align with 02_log.js
pub const DEBUG_LEVEL: Level = Level(0);
pub const INFO_LEVEL: Level = Level(5);
pub const ERROR_LEVEL: Level = Level(9);

pub struct Entry {
  pub env: String,
  pub seq: i64,
  pub time: OffsetDateTime,
  pub level: Level,
  pub func: String,
  pub message: String,
}

const BUF_LEN: usize = 32;

thread_local! {
    static LOG_BUF : RefCell<Vec<Entry>> = RefCell::new(Vec::with_capacity(BUF_LEN));
}

static RE: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"at\s+(\S+)\s+\(.*/([^/]+):(\d+):\d+\)").unwrap());

pub(crate) fn record(
  env: &str,
  seq: i64,
  level: i32,
  stack: String,
  message: String,
) {
  let func = if let Some(captures) = RE.captures(stack.as_str()) {
    let function_name = captures.get(1).unwrap().as_str();
    let file_name = captures.get(2).unwrap().as_str();
    let line_number = captures.get(3).unwrap().as_str();
    format!("{}:{}:{}", file_name, line_number, function_name)
  } else {
    stack
  };

  LOG_BUF.with(|v| {
    v.borrow_mut().push(Entry {
      env: env.to_string(),
      seq,
      time: OffsetDateTime::now_utc(),
      level: Level(level),
      func,
      message,
    })
  });
}

pub fn collect() -> Vec<Entry> {
  LOG_BUF.with(|v| v.borrow_mut().drain(..).collect())
}
