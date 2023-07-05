use dashmap::DashMap;
use once_cell::sync::Lazy;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
struct Deployment {
    environment_id: String,
    deploy_seq: i64,
    bundles: Vec<Bundle>,
    http_routes: Vec<HttpRoute>,
}

#[derive(Clone, Debug, Deserialize)]
struct Bundle {
    id: String,
    fs_path: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct HttpRoute {
    pub id: String,
    pub http_path: String,
    pub method: String,
    pub js_entry_point: String,
    pub js_export: String,
}
static GLOBAL_ROUTER: Lazy<DashMap<String, Vec<Deployment>>> =
    Lazy::new(|| DashMap::new());

pub fn match_route(
    environment_id: &str,
    func_url: &str,
    method: &str,
) -> Option<(i64, HttpRoute)> {
    if let Some(entry) = GLOBAL_ROUTER.get(environment_id) {
        let cur_deploy = entry[0].clone();
        for route in cur_deploy.http_routes.iter() {
            if route.http_path == func_url && route.method == method {
                return Some((cur_deploy.deploy_seq, route.clone()));
            }
        }
        None
    } else {
        None
    }
}

fn add_route(mut deployment: Deployment) {
    let env_id = deployment.environment_id.clone();
    deployment
        .http_routes
        .sort_by(|a, b| a.http_path.cmp(&b.http_path));
    let mut entry = GLOBAL_ROUTER.entry(env_id).or_insert_with(|| Vec::new());
    entry.insert(0, deployment.clone());
    entry.sort_by(|a, b| b.deploy_seq.cmp(&a.deploy_seq));
}
