use crate::api_mgr::ApiError;
use anyhow::{anyhow, Context, Result};
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use deno_core::url::Url;
use deno_core::{serde_v8, v8};
use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::task::LocalSet;

use crate::runtime::DarxIsolate;

use crate::utils::create_db_pool;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::runtime::Builder;
use tokio::sync::mpsc::error::SendError;
use tokio::sync::{mpsc, oneshot};

pub async fn run_server() -> Result<()> {
    let db_pool = create_db_pool();
    let worker_pool = WorkerPool::new();
    let server_state = Arc::new(ServerState {
        db_pool,
        worker_pool,
    });

    let app = Router::new()
        .route(
            "/d/api/f/:function_name",
            get(invoke_func_get).post(invoke_func_post),
        )
        .route("/c/modules", get(list_modules).post(create_module))
        .with_state(server_state);
    axum::Server::bind(&"127.0.0.1:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await?;
    Ok(())
}

async fn invoke_func_get(
    State(server_state): State<Arc<ServerState>>,
    Query(params): Query<HashMap<String, String>>,
    Path(func_name): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let db_pool = (&server_state).db_pool.clone();
    let (resp_tx, resp_rx) = oneshot::channel();
    let event = WorkerEvent::InvokeFunction {
        db_pool,
        tenant_id: tenant_dir().to_string(),
        func_name: func_name.clone(),
        params,
        resp: resp_tx,
    };
    server_state
        .worker_pool
        .send
        .send(event)
        .with_context(|| format!("failed to send event to worker pool"))?;

    let result = resp_rx.await.with_context(|| {
        format!(
            "failed to receive response from worker pool for function '{}'",
            func_name
        )
    })?;
    Ok(Json(result.unwrap()))
}

async fn invoke_func_post() {}

async fn list_modules() -> Result<Json<serde_json::Value>, ApiError> {
    todo!()
}

async fn create_module(
    Json(req): Json<CreateModuleRequest>,
) -> Result<StatusCode, ApiError> {
    // save file to tenant's directory
    let tenant_dir_path = PathBuf::from(tenant_dir());
    fs::create_dir_all(tenant_dir_path.as_path())
        .await
        .with_context(|| {
            format!(
                "failed to create directory: {}",
                tenant_dir_path.to_str().unwrap()
            )
        })?;

    let file_full_path = if let Some(dir) = req.dir {
        let mut file_dir_path = tenant_dir_path.clone();
        file_dir_path.push(&dir);
        fs::create_dir_all(file_dir_path.as_path())
            .await
            .with_context(|| {
                format!(
                    "failed to create directory: {}",
                    file_dir_path.to_str().unwrap()
                )
            })?;
        format!("{}/{}/{}", tenant_dir(), dir, req.file_name)
    } else {
        format!("{}/{}", tenant_dir(), req.file_name)
    };

    let mut f = File::create(PathBuf::from(file_full_path.as_str()))
        .await
        .with_context(|| {
            format!("failed to create module file: {}", file_full_path.as_str())
        })?;

    f.write_all(req.code.as_ref()).await.with_context(|| {
        format!("failed to write module file: {}", file_full_path.as_str())
    })?;

    Ok(StatusCode::CREATED)
}

fn tenant_dir() -> &'static str {
    "examples/tenants/1234567"
}

#[derive(Deserialize)]
struct CreateModuleRequest {
    dir: Option<String>,
    file_name: String,
    code: String,
}

struct ServerState {
    db_pool: mysql_async::Pool,
    worker_pool: WorkerPool,
}

struct WorkerPool {
    // todo: use mpmc
    send: mpsc::UnboundedSender<WorkerEvent>,
}

impl WorkerPool {
    pub fn new() -> Self {
        let (send, mut recv) = mpsc::unbounded_channel();
        let rt = Builder::new_current_thread().enable_all().build().unwrap();

        std::thread::spawn(move || {
            let local = LocalSet::new();
            local.spawn_local(async move {
                while let Some(new_event) = recv.recv().await {
                    tokio::task::spawn_local(handle_event(new_event));
                }
            });
            // This will return once all senders are dropped and all
            // spawned tasks have returned.
            rt.block_on(local);
        });
        Self { send }
    }

    pub fn send(
        &self,
        event: WorkerEvent,
    ) -> std::result::Result<(), SendError<WorkerEvent>> {
        self.send.send(event)
    }
}

#[derive(Debug)]
enum WorkerEvent {
    InvokeFunction {
        db_pool: mysql_async::Pool,
        tenant_id: String,
        func_name: String,
        params: HashMap<String, String>,
        resp: Responder<serde_json::Value>,
    },
}

type Responder<T> = oneshot::Sender<Result<T>>;

async fn handle_event(event: WorkerEvent) {
    match event {
        WorkerEvent::InvokeFunction {
            db_pool,
            tenant_id,
            func_name,
            params,
            resp,
        } => {
            // todo: use some thing real. this is just for testing
            let tenant_path =
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(tenant_dir());
            let mut isolate = DarxIsolate::new(db_pool, tenant_path);
            let func_path = format!("./api/{}.js", func_name);
            // evaluate the module here to check the syntax.
            let r = isolate.load_and_eval_module_file(&func_path).await;
            match r {
                Ok(()) => {
                    // register the function
                    println!("{}", registry_code(&func_path));
                    isolate
                        .load_and_evaluate_module_code(
                            "./registry.js",
                            registry_code(&func_path).as_str(),
                        )
                        .await
                        .unwrap();

                    let script_result = isolate
                        .js_runtime
                        .execute_script(
                            "myfoo",
                            r#"
                            handler();
                    "#,
                        )
                        .unwrap();

                    let script_result = isolate
                        .js_runtime
                        .resolve_value(script_result)
                        .await
                        .unwrap();
                    let mut handle_scope = isolate.js_runtime.handle_scope();
                    let script_result =
                        v8::Local::new(&mut handle_scope, script_result);
                    let script_result: serde_json::Value =
                        serde_v8::from_v8(&mut handle_scope, script_result)
                            .unwrap();
                    resp.send(Ok(script_result)).unwrap();
                }
                Err(e) => resp.send(Err(e)).unwrap(),
            }
        }
    }
}

fn registry_code(import_name: &str) -> String {
    format!(
        "
    import {{handler}} from \"{}\";
    globalThis.handler = handler;
    ",
        import_name
    )
}
