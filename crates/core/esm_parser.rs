use std::io::stderr;

use crate::FunctionSignatureV1;
use anyhow::{anyhow, bail, Result};
use swc_common::errors::Handler;
use swc_common::sync::Lrc;
use swc_common::{FileName, SourceMap};
use swc_ecma_ast::{Decl, DefaultDecl, Function, ModuleDecl, ModuleItem, Pat};
use swc_ecma_parser::parse_file_as_module;

// todo: handle Javascript syntax error
pub fn parse_module_export(
    file_name: &str,
    source: &str,
) -> Result<Vec<FunctionSignatureV1>> {
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

    let mut sigs = vec![];

    for item in module.body.iter() {
        match item {
            ModuleItem::ModuleDecl(decl) => match decl {
                ModuleDecl::ExportDecl(export_decl) => {
                    match &export_decl.decl {
                        Decl::Fn(fn_decl) => {
                            let params =
                                extract_fn_parameters(&fn_decl.function)?;
                            sigs.push(FunctionSignatureV1 {
                                export_name: fn_decl.ident.sym.to_string(),
                                param_names: params,
                            });
                        }
                        _ => {}
                    }
                }
                ModuleDecl::ExportDefaultDecl(export_default_decl) => {
                    match &export_default_decl.decl {
                        DefaultDecl::Fn(f) => {
                            let params = extract_fn_parameters(&f.function)?;
                            sigs.push(FunctionSignatureV1 {
                                export_name: "default".to_string(),
                                param_names: params,
                            });
                        }
                        _ => {}
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }
    Ok(sigs)
}

fn extract_fn_parameters(f: &Box<Function>) -> Result<Vec<String>> {
    let mut params = vec![];
    for p in f.params.iter() {
        match &p.pat {
            Pat::Ident(ident) => {
                params.push(ident.sym.to_string());
            }
            _ => bail!("function signature not supported"),
        }
    }
    Ok(params)
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
        export function sub(c, d) {
            return c - d;
        }
                
        export default function mul(e, f) {
            return e * f;
        }
                
        "#;
        let sigs = parse_module_export("test.js", source).unwrap();
        assert_eq!(sigs.len(), 3);
        assert_eq!(sigs[0].export_name, "add");
        assert_eq!(sigs[0].param_names, ["a", "b"]);
        assert_eq!(sigs[1].export_name, "sub");
        assert_eq!(sigs[1].param_names, ["c", "d"]);
        assert_eq!(sigs[2].export_name, "default");
        assert_eq!(sigs[2].param_names, ["e", "f"]);
    }
}
