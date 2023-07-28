use anyhow::{anyhow, Result};

use darx_core::{FunctionSignatureV1, HttpRoute};

/// Build http path from entry point and export.
/// For a pair of (entry_point, export), here are some examples:
/// - (foo.js, default)       -> foo
/// - (foo.js, bar)           -> foo.bar
/// - (foo/foo.js, default)   -> foo/foo
/// - (foo/foo.js, bar)       -> foo/foo.bar
pub fn build_route(
    prefix: Option<&str>,
    entry_point: &str,
    func_sig: &FunctionSignatureV1,
) -> Result<HttpRoute> {
    let path = if entry_point.ends_with(".js") {
        entry_point.strip_suffix(".js").unwrap()
    } else if entry_point.ends_with(".ts") {
        entry_point.strip_suffix(".ts").unwrap()
    } else if entry_point.ends_with(".mjs") {
        entry_point.strip_suffix(".mjs").unwrap()
    } else {
        return Err(anyhow!("Invalid entry point: {}", entry_point));
    };
    let path = if let Some(prefix) = prefix {
        path.strip_prefix(prefix).unwrap_or(path)
    } else {
        path
    };
    let path = if func_sig.export_name == "default" {
        path.to_string()
    } else {
        format!("{}.{}", path, func_sig.export_name)
    };
    Ok(HttpRoute {
        http_path: path,
        method: "POST".to_string(),
        js_entry_point: entry_point.to_string(),
        js_export: func_sig.export_name.to_string(),
        func_sig_version: 1,
        func_sig: func_sig.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_build_route_default_export() {
        let entry_point = "foo.js";
        let sig = FunctionSignatureV1 {
            export_name: "default".to_string(),
            param_names: vec![],
        };
        assert_eq!(
            build_route(None, entry_point, &sig).unwrap().http_path,
            "foo"
        );

        assert_eq!(
            build_route(Some("functions/"), entry_point, &sig)
                .unwrap()
                .http_path,
            "foo"
        );
    }

    #[test]
    fn test_build_route_custom_export() {
        let entry_point = "foo.js";
        let sig = FunctionSignatureV1 {
            export_name: "bar".to_string(),
            param_names: vec![],
        };
        assert_eq!(
            build_route(None, entry_point, &sig).unwrap().http_path,
            "foo.bar"
        );

        assert_eq!(
            build_route(Some("functions/"), entry_point, &sig)
                .unwrap()
                .http_path,
            "foo.bar"
        );
    }

    #[test]
    fn test_build_route_nested_default_export() {
        let entry_point = "foo/bar.js";
        let sig = FunctionSignatureV1 {
            export_name: "default".to_string(),
            param_names: vec![],
        };
        assert_eq!(
            build_route(None, entry_point, &sig).unwrap().http_path,
            "foo/bar"
        );

        assert_eq!(
            build_route(Some("functions/"), entry_point, &sig)
                .unwrap()
                .http_path,
            "foo/bar"
        );
    }

    #[test]
    fn test_build_route_nested_custom_export() {
        let entry_point = "foo/bar.js";
        let sig = FunctionSignatureV1 {
            export_name: "baz".to_string(),
            param_names: vec![],
        };
        assert_eq!(
            build_route(None, entry_point, &sig).unwrap().http_path,
            "foo/bar.baz"
        );
        assert_eq!(
            build_route(Some("functions/"), entry_point, &sig)
                .unwrap()
                .http_path,
            "foo/bar.baz"
        );
    }

    #[test]
    fn test_build_route_invalid_entry_point() {
        let entry_point = "foo.html";
        let sig = FunctionSignatureV1 {
            export_name: "default".to_string(),
            param_names: vec![],
        };
        assert!(build_route(None, entry_point, &sig).is_err());
        assert!(build_route(Some("functions/"), entry_point, &sig).is_err());
    }
}
