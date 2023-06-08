use darx_db::ConnectionPool;
use darx_isolate_runtime::DarxIsolate;
use deno_core::{serde_v8, v8};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::runtime::Builder;
use tokio::sync::mpsc::error::SendError;
use tokio::sync::{mpsc, oneshot};
use tokio::task::LocalSet;

pub struct WorkerPool {
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
    ) -> Result<(), SendError<WorkerEvent>> {
        self.send.send(event)
    }
}

pub enum WorkerEvent {
    InvokeFunction {
        project_id: String,
        bundle_dir: PathBuf,
        func_name: String,
        params: Box<serde_json::value::RawValue>,
        resp: Responder<serde_json::Value>,
    },
}

type Responder<T> = oneshot::Sender<anyhow::Result<T>>;

async fn handle_event(event: WorkerEvent) {
    match event {
        WorkerEvent::InvokeFunction {
            project_id,
            bundle_dir: tenant_dir,
            func_name,
            params,
            resp,
        } => {
            // todo: use some thing real. this is just for testing
            let project_dir = PathBuf::from(tenant_dir);
            let mut isolate =
                DarxIsolate::new(project_id.as_str(), project_dir);
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
                            format!("handler({});", params.get()),
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
