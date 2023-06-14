use anyhow::{anyhow, Result};
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

/// Build routes from meta file.
/// The result is sorted by [`http_path`] of the [`Route`].
pub async fn build_routes(meta_file: &str) -> Result<Vec<Route>> {
    let mut file = File::open(meta_file).await?;
    let mut buf = String::new();
    file.read_to_string(&mut buf).await?;
    let meta: serde_json::Value = serde_json::from_str(&buf)?;
    let outputs = meta
        .get("outputs")
        .ok_or_else(|| anyhow!("No outputs found"))?
        .as_object()
        .ok_or_else(|| anyhow!("Outputs is not an object"))?;

    let mut routes = vec![];
    for (_, output) in outputs.iter() {
        let output = output
            .as_object()
            .ok_or_else(|| anyhow!("Output is not an object"))?;
        let nbytes = output
            .get("bytes")
            .ok_or_else(|| anyhow!("bytes not found"))?
            .as_i64()
            .ok_or_else(|| anyhow!("bytes is not a i64"))?;

        if nbytes == 0 {
            continue;
        }

        let entry_point = output
            .get("entryPoint")
            .ok_or_else(|| anyhow!("entryPoint not found"))?
            .as_str()
            .ok_or_else(|| anyhow!("entryPoint is not a string"))?
            .to_string();

        let exports = output
            .get("exports")
            .ok_or_else(|| anyhow!("exports not found"))?
            .as_array()
            .ok_or_else(|| anyhow!("exports is not an array"))?
            .iter()
            .map(|export| {
                export
                    .as_str()
                    .ok_or_else(|| anyhow!("export is not a string"))
                    .map(|s| s.to_string())
            })
            .collect::<Result<Vec<_>>>()?;
        for export in exports.iter() {
            let http_path = build_path(&entry_point, &export)?;
            routes.push(Route {
                http_path,
                js_entry_point: entry_point.clone(),
                js_export: export.clone(),
            })
        }
    }
    routes.sort_by(|a, b| a.http_path.cmp(&b.http_path));
    Ok(routes)
}

#[derive(Debug, PartialEq)]
pub struct Route {
    pub http_path: String,
    /// `js_entry_point` is used to find the js file.
    pub js_entry_point: String,
    pub js_export: String,
}

/// Build http path from entry point and export.
/// For a pair of (entry_point, export), here are some examples:
/// - (foo.js, default)       -> foo
/// - (foo.js, bar)           -> foo.bar
/// - (foo/foo.js, default)   -> foo/foo
/// - (foo/foo.js, bar)       -> foo/foo.bar
fn build_path(entry_point: &str, export: &str) -> Result<String> {
    let path = if entry_point.ends_with(".js") {
        entry_point.strip_suffix(".js").unwrap()
    } else if entry_point.ends_with(".ts") {
        entry_point.strip_suffix(".ts").unwrap()
    } else if entry_point.ends_with(".mjs") {
        entry_point.strip_suffix(".mjs").unwrap()
    } else {
        return Err(anyhow!("Invalid entry point: {}", entry_point));
    };
    let path = if export == "default" {
        path.to_string()
    } else {
        format!("{}.{}", path, export)
    };
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_path_default_export() {
        let entry_point = "foo.js";
        let export = "default";
        assert_eq!(build_path(entry_point, export).unwrap(), "foo");
    }

    #[test]
    fn test_build_path_custom_export() {
        let entry_point = "foo.js";
        let export = "bar";
        assert_eq!(build_path(entry_point, export).unwrap(), "foo.bar");
    }

    #[test]
    fn test_build_path_nested_default_export() {
        let entry_point = "foo/bar.js";
        let export = "default";
        assert_eq!(build_path(entry_point, export).unwrap(), "foo/bar");
    }

    #[test]
    fn test_build_path_nested_custom_export() {
        let entry_point = "foo/bar.js";
        let export = "baz";
        assert_eq!(build_path(entry_point, export).unwrap(), "foo/bar.baz");
    }

    #[test]
    fn test_build_path_invalid_entry_point() {
        let entry_point = "foo.html";
        let export = "default";
        assert!(build_path(entry_point, export).is_err());
    }

    #[tokio::test]
    async fn test_api_route() {
        let meta_file_path = format!(
            "{}/examples/projects/1234567/functions/__output/meta.json",
            env!("CARGO_MANIFEST_DIR")
        );
        let routes = build_routes(meta_file_path.as_str())
            .await
            .expect("Failed to build routes");

        assert_eq!(routes.len(), 5);
        assert_eq!(
            routes,
            vec![
                Route {
                    http_path: "foo".to_string(),
                    js_entry_point: "foo.js".to_string(),
                    js_export: "default".to_string(),
                },
                Route {
                    http_path: "foo.bar".to_string(),
                    js_entry_point: "foo.js".to_string(),
                    js_export: "bar".to_string(),
                },
                Route {
                    http_path: "foo.foo".to_string(),
                    js_entry_point: "foo.js".to_string(),
                    js_export: "foo".to_string(),
                },
                Route {
                    http_path: "foo.fooWithParam".to_string(),
                    js_entry_point: "foo.js".to_string(),
                    js_export: "fooWithParam".to_string(),
                },
                Route {
                    http_path: "foo/foo".to_string(),
                    js_entry_point: "foo/foo.js".to_string(),
                    js_export: "default".to_string(),
                }
            ]
        )
    }
}
