use criterion::{
    black_box, criterion_group, criterion_main, BenchmarkId, Criterion,
};
use darx_isolate_runtime::DarxIsolate;
use deno_core::{serde_v8, v8};
use std::path::PathBuf;

const ENV_ID: &str = "cljb3ovlt0002e38vwo0xi5ge";
const DEPLOY_SEQ: i64 = 99;

async fn raw() {
    let mut darx_runtime = DarxIsolate::new(
        ENV_ID,
        DEPLOY_SEQ,
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(format!("tests/data/{}/{}", ENV_ID, DEPLOY_SEQ)),
    );

    darx_runtime
        .load_and_eval_module_file("bench.js")
        .await
        .expect("bench.js should not result an error");
}

async fn snapshot() {
    let mut _darx_runtime = DarxIsolate::new_with_snapshot(
        ENV_ID,
        DEPLOY_SEQ,
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(format!("tests/data/{}/{}", ENV_ID, DEPLOY_SEQ)),
    );
}

fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("create_isolate");
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        DarxIsolate::build_snapshot(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join(format!("tests/data/{}/{}", ENV_ID, DEPLOY_SEQ)),
        )
        .await
        .unwrap();
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
