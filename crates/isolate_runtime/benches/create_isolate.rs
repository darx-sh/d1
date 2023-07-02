use criterion::{
    black_box, criterion_group, criterion_main, BenchmarkId, Criterion,
};
use darx_isolate_runtime::DarxIsolate;
use deno_core::{serde_v8, v8};
use std::path::PathBuf;
use tokio::fs;
use tokio::time::Instant;

const ENV_ID: &str = "cljb3ovlt0002e38vwo0xi5ge";
const DEPLOY_SEQ: i64 = 99;

fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("create_isolate");
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        build_snapshot().await;
    });

    group.bench_function(BenchmarkId::new("raw", "raw"), |b| {
        b.to_async(&rt).iter(|| black_box(raw()))
    });
    group.bench_function(BenchmarkId::new("snapshot", "snapshot"), |b| {
        b.to_async(&rt).iter(|| black_box(snapshot()))
    });

    group.finish();
}

criterion_group!(create_isolate, bench);
criterion_main!(create_isolate);

async fn raw() {
    let mut isolate = DarxIsolate::new(
        ENV_ID,
        DEPLOY_SEQ,
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
    let snapshot_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        format!("tests/data/{}/{}/{}", ENV_ID, DEPLOY_SEQ, SNAPSHOT_FILE),
    );
    let snapshot = fs::read(&snapshot_path).await.unwrap();

    let mut isolate = DarxIsolate::new_with_snapshot(
        ENV_ID,
        DEPLOY_SEQ,
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(format!("tests/data/{}/{}", ENV_ID, DEPLOY_SEQ)),
        snapshot.into_boxed_slice(),
    )
    .await;

    call_foo(&mut isolate.js_runtime).await;
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
