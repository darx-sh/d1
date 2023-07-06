use anyhow::{anyhow, Result};
use darx_api::HttpRoute;

/// Build http path from entry point and export.
/// For a pair of (entry_point, export), here are some examples:
/// - (foo.js, default)       -> foo
/// - (foo.js, bar)           -> foo.bar
/// - (foo/foo.js, default)   -> foo/foo
/// - (foo/foo.js, bar)       -> foo/foo.bar
pub fn build_route(entry_point: &str, export: &str) -> Result<HttpRoute> {
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
    Ok(HttpRoute {
        http_path: path,
        method: "POST".to_string(),
        js_entry_point: entry_point.to_string(),
        js_export: export.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_route_default_export() {
        let entry_point = "foo.js";
        let export = "default";
        assert_eq!(build_route(entry_point, export).unwrap().http_path, "foo");
    }

    #[test]
    fn test_build_route_custom_export() {
        let entry_point = "foo.js";
        let export = "bar";
        assert_eq!(
            build_route(entry_point, export).unwrap().http_path,
            "foo.bar"
        );
    }

    #[test]
    fn test_build_route_nested_default_export() {
        let entry_point = "foo/bar.js";
        let export = "default";
        assert_eq!(
            build_route(entry_point, export).unwrap().http_path,
            "foo/bar"
        );
    }

    #[test]
    fn test_build_route_nested_custom_export() {
        let entry_point = "foo/bar.js";
        let export = "baz";
        assert_eq!(
            build_route(entry_point, export).unwrap().http_path,
            "foo/bar.baz"
        );
    }

    #[test]
    fn test_build_route_invalid_entry_point() {
        let entry_point = "foo.html";
        let export = "default";
        assert!(build_route(entry_point, export).is_err());
    }

    // #[tokio::test]
    // async fn test_api_route() {
    //     let meta_file_path = format!(
    //         "{}/examples/projects/1234567/functions/__output/meta.json",
    //         env!("CARGO_MANIFEST_DIR")
    //     );
    //     let routes = build_routes(meta_file_path.as_str())
    //         .await
    //         .expect("Failed to build routes");
    //
    //     assert_eq!(routes.len(), 5);
    //     assert_eq!(
    //         routes,
    //         vec![
    //             Route {
    //                 http_path: "foo".to_string(),
    //                 js_entry_point: "foo.js".to_string(),
    //                 js_export: "default".to_string(),
    //             },
    //             Route {
    //                 http_path: "foo.bar".to_string(),
    //                 js_entry_point: "foo.js".to_string(),
    //                 js_export: "bar".to_string(),
    //             },
    //             Route {
    //                 http_path: "foo.foo".to_string(),
    //                 js_entry_point: "foo.js".to_string(),
    //                 js_export: "foo".to_string(),
    //             },
    //             Route {
    //                 http_path: "foo.fooWithParam".to_string(),
    //                 js_entry_point: "foo.js".to_string(),
    //                 js_export: "fooWithParam".to_string(),
    //             },
    //             Route {
    //                 http_path: "foo/foo".to_string(),
    //                 js_entry_point: "foo/foo.js".to_string(),
    //                 js_export: "default".to_string(),
    //             }
    //         ]
    //     )
    // }
}
