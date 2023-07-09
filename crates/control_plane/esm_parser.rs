use std::io::stderr;

use anyhow::{anyhow, Result};
use swc_common::{FileName, SourceMap};
use swc_common::errors::Handler;
use swc_common::sync::Lrc;
use swc_ecma_ast::{Decl, DefaultDecl, ModuleDecl, ModuleItem};
use swc_ecma_parser::parse_file_as_module;

// todo: handle Javascript syntax error
pub fn parse_module_export(
    file_name: &str,
    source: &str,
) -> Result<Vec<String>> {
    let cm: Lrc<SourceMap> = Default::default();
    let handler =
        Handler::with_emitter_writer(Box::new(stderr()), Some(cm.clone()));
    let fm = cm.new_source_file(
        FileName::Custom(file_name.to_string()),
        source.to_string(),
    );

    let module = parse_file_as_module(
        &fm,
        Default::default(),
        Default::default(),
        None,
        &mut vec![],
    )
    .map_err(|err| {
        err.into_diagnostic(&handler).emit();
        anyhow!("parse_file_as_module error")
    })?;

    let mut js_exports = vec![];

    for item in module.body.iter() {
        match item {
            ModuleItem::ModuleDecl(decl) => match decl {
                ModuleDecl::ExportDecl(export_decl) => {
                    match &export_decl.decl {
                        Decl::Fn(fn_decl) => {
                            js_exports.push(fn_decl.ident.sym.to_string());
                        }
                        _ => {}
                    }
                }
                ModuleDecl::ExportDefaultDecl(export_default_decl) => {
                    match &export_default_decl.decl {
                        DefaultDecl::Fn(_) => {
                            js_exports.push("default".to_string());
                        }
                        _ => {}
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }
    Ok(js_exports)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_func_export() {
        let source = r#"
        export function add(a, b) {
            return a + b;
        }
        export function sub(a, b) {
            return a - b;
        }
                
        export default function mul(a, b) {
            return a * b;
        }
                
        "#;
        let exports = parse_module_export("test.js", source).unwrap();
        assert_eq!(exports.len(), 3);
        assert_eq!(exports[0], "add");
        assert_eq!(exports[1], "sub");
        assert_eq!(exports[2], "default");
    }
}
