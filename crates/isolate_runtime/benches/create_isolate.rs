use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::DerefMut;
use std::path::PathBuf;
use std::rc::Rc;

use criterion::{
  black_box, criterion_group, criterion_main, BenchmarkId, Criterion,
};
use deno_core::{serde_v8, v8, JsRealm};
use tokio::fs;
use tokio::time::Instant;

use darx_isolate_runtime::DarxIsolate;

const ENV_ID: &str = "8nvcym53y8d2";
const DEPLOY_SEQ: i64 = 99;

fn bench(c: &mut Criterion) {
  let mut group = c.benchmark_group("create_isolate");
  let rt = tokio::runtime::Builder::new_current_thread()
    .enable_all()
    .build()
    .unwrap();
  rt.block_on(async {
    build_snapshot().await;
  });

  let mut isolate = rt.block_on(async {
    let snapshot = read_snapshot().await;
    new_isolate(snapshot).await
  });

  let isolate = Rc::new(RefCell::new(isolate));

  group.bench_function(BenchmarkId::new("raw", "raw"), |b| {
    b.to_async(&rt).iter(|| black_box(raw()))
  });
  group.bench_function(BenchmarkId::new("snapshot", "snapshot"), |b| {
    b.to_async(&rt).iter(|| black_box(snapshot()))
  });
  group.bench_function(BenchmarkId::new("realm", "realm"), |b| {
    b.to_async(&rt).iter(|| black_box(realm(isolate.clone())))
  });
  group.bench_function(BenchmarkId::new("globalRealm", "globalRealm"), |b| {
    b.to_async(&rt)
      .iter(|| black_box(globalRealm(isolate.clone())))
  });
  group.finish();
}

criterion_group!(create_isolate, bench);
criterion_main!(create_isolate);

async fn raw() {
  let mut isolate = DarxIsolate::new(
    ENV_ID,
    DEPLOY_SEQ,
    &Default::default(),
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
      .join(format!("tests/data/{}/{}", ENV_ID, DEPLOY_SEQ)),
  );

  isolate
    .load_and_eval_module_file("bench.js")
    .await
    .expect("bench.js should not result an error");

  call_foo(&mut isolate.js_runtime).await;
}

const SNAPSHOT_FILE: &str = "SNAPSHOT.bin";

async fn snapshot() {
  let snapshot = read_snapshot().await;
  let mut isolate = new_isolate(snapshot).await;
  call_foo(&mut isolate.js_runtime).await;
}

async fn realm(isolate: Rc<RefCell<DarxIsolate>>) {
  let mut isolate = isolate.borrow_mut();
  let js_runtime = &mut isolate.js_runtime;
  let v8_isolate = js_runtime.v8_isolate();
  let realm: JsRealm;
  {
    let mut handle_scope = v8::HandleScope::new(v8_isolate);
    let context = v8::Context::new(&mut handle_scope);
    let mut context_scope = v8::ContextScope::new(&mut handle_scope, context);
    let global = v8::Global::new(&mut context_scope, context);
    realm = JsRealm::new(global);
  }

  let script_result = realm.execute_script(v8_isolate, "run", "foo()").unwrap();
  let mut handle_scope = realm.handle_scope(v8_isolate);
  let script_result = v8::Local::new(&mut handle_scope, script_result);
  let script_result: i32 =
    serde_v8::from_v8(&mut handle_scope, script_result).unwrap();
  assert_eq!(2, script_result);
}

async fn globalRealm(isolate: Rc<RefCell<DarxIsolate>>) {
  let mut isolate = isolate.borrow_mut();
  let js_runtime = &mut isolate.js_runtime;
  let v8_isolate = js_runtime.v8_isolate();
  let script_result = js_runtime.execute_script("run", "foo()").unwrap();
  let mut handle_scope = js_runtime.handle_scope();
  let script_result = v8::Local::new(&mut handle_scope, script_result);
  let script_result: i32 =
    serde_v8::from_v8(&mut handle_scope, script_result).unwrap();
  assert_eq!(2, script_result);
}

async fn call_foo(js_runtime: &mut deno_core::JsRuntime) {
  let script_result = js_runtime.execute_script("run", "foo()").unwrap();
  let script_result = js_runtime.resolve_value(script_result).await.unwrap();
  let mut handle_scope = js_runtime.handle_scope();
  let script_result = v8::Local::new(&mut handle_scope, script_result);
  let script_result: i32 =
    serde_v8::from_v8(&mut handle_scope, script_result).unwrap();
  assert_eq!(2, script_result);
}

async fn build_snapshot() {
  let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .join(format!("tests/data/{}/{}", ENV_ID, DEPLOY_SEQ));
  let mut js_runtime = DarxIsolate::prepare_snapshot(&path).await.unwrap();

  //TODO should load all js files
  let module_id = js_runtime
    .load_side_module(
      &deno_core::resolve_path("bench.js", &path).unwrap(),
      None,
    )
    .await
    .unwrap();

  let receiver = js_runtime.mod_evaluate(module_id);
  js_runtime.run_event_loop(false).await.unwrap();
  let _ = receiver.await.unwrap();

  let mut mark = Instant::now();
  let snapshot = js_runtime.snapshot();
  let snapshot_slice: &[u8] = &snapshot;
  println!(
    "Snapshot size: {}, took {:#?}",
    snapshot_slice.len(),
    Instant::now().saturating_duration_since(mark)
  );

  mark = Instant::now();

  let snapshot_path = format!("{}/{}", path.display(), SNAPSHOT_FILE);

  fs::write(&snapshot_path, snapshot_slice).await.unwrap();

  println!(
    "Snapshot written, took: {:#?} ({})",
    Instant::now().saturating_duration_since(mark),
    &snapshot_path,
  );
}

async fn read_snapshot() -> Vec<u8> {
  let snapshot_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
    "tests/data/{}/{}/{}",
    ENV_ID, DEPLOY_SEQ, SNAPSHOT_FILE
  ));
  let snapshot = fs::read(&snapshot_path).await.unwrap();
  snapshot
}

async fn new_isolate(snapshot: Vec<u8>) -> DarxIsolate {
  DarxIsolate::new_with_snapshot(
    ENV_ID,
    DEPLOY_SEQ,
    &Default::default(),
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
      .join(format!("tests/data/{}/{}", ENV_ID, DEPLOY_SEQ)),
    snapshot.into_boxed_slice(),
  )
  .await
}
