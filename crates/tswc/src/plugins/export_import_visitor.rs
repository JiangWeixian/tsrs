use std::collections::HashMap;

use log::debug;
use swc_core::ecma::ast::{
  self, ImportPhase, ImportSpecifier as SWCImportSpecifier, ModuleExportName, NamedExport,
};
use swc_core::ecma::visit::{VisitMut, VisitMutWith};

use crate::compiler::{Module, ModuleGraph, ResolveModuleOptions};
use crate::utils::{ExportSpecifier, ImportSpecifier, ImportType};
use lazy_static::lazy_static;

lazy_static! {
  pub static ref TS_EXTS: Vec<&'static str> = vec!["ts", "tsx", "mts", "cts", "mtsx", "ctsx"];
  pub static ref JSX_EXTS: Vec<&'static str> = vec!["tsx", "jsx", "mtsx", "ctsx", "mjsx", "cjsx"];
  pub static ref NOT: i32 = -1;
  pub static ref NOT_BECAUSE_META: i32 = -2;
  pub static ref DEFAULT_EXPORT: &'static str = "default";
  pub static ref DEFAULT_EXPORT_LEN: i32 = 7;
  pub static ref BRACKET_LEFT: &'static str = "(";
  pub static ref SEMI: &'static str = ";";
  pub static ref SEMI_UNICODE: u16 = SEMI.encode_utf16().next().unwrap();
  pub static ref EXPORT_LEN: i32 = 6;
}

pub struct ImportExportVisitor<'a> {
  pub context: String,
  pub imports: Vec<ImportSpecifier>,
  pub module_graph: &'a mut ModuleGraph,
  pub exports: Vec<ExportSpecifier>,
  pub facade: bool,
  pub local_idents: HashMap<String, (String, String)>,
  pub has_module_syntax: bool,
}

impl<'a> ImportExportVisitor<'a> {
  pub fn new(module_graph: &'a mut ModuleGraph, context: String) -> Self {
    Self {
      imports: vec![],
      exports: vec![],
      facade: false,
      has_module_syntax: false,
      module_graph,
      context,
      local_idents: HashMap::default(),
    }
  }
}

// import
impl<'a> ImportExportVisitor<'a> {
  fn add_import(&mut self, import: ImportSpecifier) -> Option<&mut Module> {
    let specifier = import.src.clone();
    debug!(target: "tswc", "add import {:?} {:?}", specifier, self.context);
    self.imports.push(import);
    let m = self.module_graph.resolve_module(ResolveModuleOptions {
      specifier,
      context: self.context.clone(),
      ..Default::default()
    });
    m
  }

  fn parse_import(&mut self, import: &mut ast::ImportDecl) {
    // import type { a } from 'b'
    // import a, { type b } from 'b'
    if import.type_only {
      return;
    }

    // import 'b'
    if import.specifiers.is_empty() {
      let name = import.src.value.to_string();
      self.add_import(ImportSpecifier {
        src: Some(name),
        t: ImportType::Static,
      });
      return;
    }

    // import { type c } from 'b'
    let is_all_type_import = import.specifiers.iter().all(|specifier| match specifier {
      ast::ImportSpecifier::Named(named) => named.is_type_only,
      _ => false,
    });
    if is_all_type_import {
      return;
    }

    for spec in &import.specifiers {
      let src = import.src.value.to_string();
      match spec {
        SWCImportSpecifier::Named(s) => {
          self.local_idents.insert(
            s.local.sym.to_string(),
            (
              src.clone(),
              match &s.imported {
                Some(n) => match &n {
                  ModuleExportName::Ident(n) => n.sym.to_string(),
                  ModuleExportName::Str(n) => n.value.to_string(),
                },
                None => s.local.sym.to_string(),
              },
            ),
          );
        }
        SWCImportSpecifier::Namespace(s) => {
          self
            .local_idents
            .insert(s.local.sym.to_string(), (src.clone(), "*".to_string()));
        }
        SWCImportSpecifier::Default(s) => {
          self.local_idents.insert(
            s.local.sym.to_string(),
            (src.clone(), "default".to_string()),
          );
        }
      }
    }

    let first_specifier = &import.specifiers[0];
    match first_specifier {
      // import a from 'bbbb'
      // import * as all from 'b'
      // import { a, b } from 'b'
      // import a, { a, b } from 'b'
      ast::ImportSpecifier::Default(_)
      | ast::ImportSpecifier::Named(_)
      | ast::ImportSpecifier::Namespace(_) => {
        let name = import.src.value.to_string();
        let mut t: Option<ImportType> = None;

        match import.phase {
          ImportPhase::Defer => {
            // import defer a from './xxx'
            // TODO: wait `es-module-lexer` support this case
          }
          ImportPhase::Evaluation => {
            // import a from './xxx'
            t = Some(ImportType::Static);
          }
          ImportPhase::Source => {
            // import source a from './xxx'
            t = Some(ImportType::StaticSourcePhase);
          }
        }

        // Replace src with resolver's result
        if t.is_some() {
          let m = self.add_import(ImportSpecifier {
            src: Some(name),
            t: t.unwrap(),
          });
          if let Some(v) = m.and_then(|f| f.with_ext()) {
            import.src = Box::new(ast::Str::from(v));
          }
        }
      }
    }
  }
}

// export
impl<'a> ImportExportVisitor<'a> {
  fn add_export(&mut self, export: ExportSpecifier) -> Option<&mut Module> {
    let specifier = export.src.clone();
    self.exports.push(export);
    let m = self.module_graph.resolve_module(ResolveModuleOptions {
      specifier,
      context: self.context.clone(),
      ..Default::default()
    });
    m
  }

  fn add_export_from_ident(&mut self, ident: &ast::Ident) {
    let name = ident.sym.to_string();
    self.add_export(ExportSpecifier {
      n: name.clone(),
      ln: Some(name),
      src: None,
      wildcard: false,
    });
  }

  fn parse_export_spec(
    &mut self,
    specifier: &ast::ExportSpecifier,
    export_named: &mut ast::NamedExport,
  ) -> (bool, Option<&mut Module>) {
    match specifier {
      ast::ExportSpecifier::Named(named) => {
        // skip type
        if named.is_type_only {
          return (false, None);
        }

        let mut is_renamed = false;
        let name = if let Some(exported) = &named.exported {
          // export { a as b }
          is_renamed = true;
          match exported {
            ast::ModuleExportName::Ident(ident) => ident.sym.to_string(),
            // export { 'a' as 'b' }
            ast::ModuleExportName::Str(str) => str.value.to_string(),
          }
        } else {
          match &named.orig {
            // export { a }
            ast::ModuleExportName::Ident(ident) => ident.sym.to_string(),
            // export { "a" }
            ast::ModuleExportName::Str(str) => str.value.to_string(),
          }
        };

        let origin_name;
        if is_renamed {
          match &named.orig {
            ast::ModuleExportName::Ident(ident) => {
              origin_name = Some(ident.sym.to_string());
            }
            // export { 'a' as 'b' }
            ast::ModuleExportName::Str(str) => {
              origin_name = Some(str.value.to_string());
            }
          }
        } else {
          origin_name = Some(name.clone());
        }

        let src_str: Option<String> = if let Some(src) = &export_named.src {
          Some(src.value.to_string())
        } else if let Some((src, orig)) = self.local_idents.get(&origin_name.clone().unwrap()) {
          Some(src.into())
        } else {
          None
        };

        debug!(target: "tswc", "add export {:?} {:?}", src_str, self.context);
        let m = self.add_export(ExportSpecifier {
          n: name,
          ln: origin_name,
          src: src_str,
          wildcard: false,
        });

        return (true, m);
      }
      // export v from 'm'
      // current not support
      ast::ExportSpecifier::Default(_) => {
        return (false, None);
      }
      // export * as a from 'b'
      ast::ExportSpecifier::Namespace(namespace) => {
        if let ast::ModuleExportName::Ident(ident) = &namespace.name {
          let name = ident.sym.to_string();
          let src_str: Option<String> = if let Some(src) = &export_named.src {
            Some(src.value.to_string())
          } else {
            None
          };
          let m = self.add_export(ExportSpecifier {
            n: name,
            ln: None,
            src: src_str,
            wildcard: false,
          });
          return (true, m);
        }
        return (false, None);
      }
    }
  }

  fn parse_named_export(&mut self, export: &mut ast::NamedExport) -> bool {
    // export type { a } from 'b'
    // export type * as a from 'b'
    if export.type_only {
      return false;
    }

    // export { type c } from 'b'
    let is_all_type_export = export.specifiers.iter().all(|specifier| match specifier {
      ast::ExportSpecifier::Named(named) => named.is_type_only,
      _ => false,
    });
    if is_all_type_export {
      return false;
    }

    let mut is_need_add_import = false;
    let mut resolved_src: Option<String> = None;
    let specifiers = &export.specifiers.clone();
    for specifier in specifiers {
      let (need_add_import, m) = self.parse_export_spec(specifier, export);
      if resolved_src.is_none() {
        resolved_src = m.and_then(|f| f.with_ext());
      }
      if need_add_import && !is_need_add_import {
        is_need_add_import = true;
      }
    }

    if let Some(v) = resolved_src {
      export.src = Some(Box::new(ast::Str::from(v)));
    }
    return is_need_add_import;
  }

  fn parse_default_export_expr(&mut self, _export: &ast::ExportDefaultExpr) {
    let name = DEFAULT_EXPORT.to_string();
    self.add_export(ExportSpecifier {
      n: name,
      ln: None,
      src: None,
      wildcard: false,
    });
  }

  fn parse_export_decl(&mut self, export: &ast::ExportDecl) -> bool {
    let mut need_eager_return = false;
    match &export.decl {
      ast::Decl::Class(decl) => self.add_export_from_ident(&decl.ident),
      ast::Decl::Fn(decl) => self.add_export_from_ident(&decl.ident),
      ast::Decl::Var(decl) => {
        decl.decls.iter().for_each(|decl| {
          // support export const a = 1, b = 2
          match &decl.name {
            ast::Pat::Ident(ident) => {
              let name = ident.sym.to_string();
              self.add_export(ExportSpecifier {
                n: name.clone(),
                ln: Some(name),
                src: None,
                wildcard: false,
              });
            }
            ast::Pat::Object(pat) => {
              pat.props.iter().for_each(|prop| {
                match &prop {
                  // export const { a, b } = {}
                  ast::ObjectPatProp::Assign(assign) => {
                    let ident = &assign.key;
                    let name = ident.sym.to_string();
                    self.add_export(ExportSpecifier {
                      n: name.clone(),
                      ln: Some(name),
                      src: None,
                      wildcard: false,
                    });
                  }
                  ast::ObjectPatProp::KeyValue(kv) => {
                    match kv.value.as_ref() {
                      // FIXME: es-module-lexer parse this case will get name:`a`, not `b`, it's a bug.
                      ast::Pat::Ident(ident) => {
                        // only support value is ident
                        let name = ident.sym.to_string();
                        self.add_export(ExportSpecifier {
                          n: name.clone(),
                          ln: Some(name),
                          src: None,
                          wildcard: false,
                        });
                      }
                      _ => {
                        // Not support
                      }
                    }
                  }
                  // Not support case: export const { a, ...b } = {}
                  // es-module-lexer not support find the `b` index
                  ast::ObjectPatProp::Rest(_) => {}
                }
              })
            }
            ast::Pat::Array(pat) => {
              pat.elems.iter().for_each(|elm| {
                if elm.is_some() {
                  // only support export const [a, b] = []
                  if let ast::Pat::Ident(ident) = &elm.as_ref().unwrap() {
                    let name = ident.sym.to_string();
                    self.add_export(ExportSpecifier {
                      n: name.clone(),
                      ln: Some(name),
                      src: None,
                      wildcard: false,
                    });
                  }
                }
              })
            }
            _ => {}
          }
        })
      }
      ast::Decl::Using(_) => {}
      ast::Decl::TsEnum(decl) => {
        let name = decl.id.sym.to_string();
        self.add_export(ExportSpecifier {
          n: name.clone(),
          ln: Some(name),
          src: None,
          wildcard: false,
        });
      }
      ast::Decl::TsModule(decl) => {
        if let ast::TsModuleName::Ident(ident) = &decl.id {
          let name = ident.sym.to_string();
          self.add_export(ExportSpecifier {
            n: name.clone(),
            ln: Some(name),
            src: None,
            wildcard: false,
          });
        }
        // do not visit import / export within namespace
        need_eager_return = true;
      }
      ast::Decl::TsInterface(_) => {}
      ast::Decl::TsTypeAlias(_) => {}
    }
    need_eager_return
  }

  fn parse_export_default_decl(&mut self, export: &ast::ExportDefaultDecl) {
    match &export.decl {
      // export default class A {}
      // export default class {}
      ast::DefaultDecl::Class(decl) => {
        if let Some(ident) = &decl.ident {
          let origin_name = ident.sym.to_string();
          self.add_export(ExportSpecifier {
            n: DEFAULT_EXPORT.to_string(),
            ln: Some(origin_name),
            src: None,
            wildcard: false,
          });
        } else {
          let name = DEFAULT_EXPORT.to_string();
          self.add_export(ExportSpecifier {
            n: name,
            ln: None,
            src: None,
            wildcard: false,
          });
        }
      }
      // export default function A() {}
      // export default function() {}
      ast::DefaultDecl::Fn(decl) => {
        if let Some(ident) = &decl.ident {
          let origin_name = ident.sym.to_string();
          self.add_export(ExportSpecifier {
            n: DEFAULT_EXPORT.to_string(),
            ln: Some(origin_name),
            src: None,
            wildcard: false,
          });
        } else {
          let name = DEFAULT_EXPORT.to_string();
          self.add_export(ExportSpecifier {
            n: name.clone(),
            ln: None,
            src: None,
            wildcard: false,
          });
        }
      }
      ast::DefaultDecl::TsInterfaceDecl(_) => {}
    }
  }
}

// utils
impl<'a> ImportExportVisitor<'a> {
  fn detect_facade(&mut self, module: &mut ast::Module) {
    let mut is_facade = true;
    for item in module.body.iter() {
      match item {
        ast::ModuleItem::ModuleDecl(decl) => {
          match decl {
            // import ...
            ast::ModuleDecl::Import(_) => {
              continue;
            }
            // e.g. export const a = 1
            ast::ModuleDecl::ExportDecl(item) => {
              match item.decl {
                // export interface A {}
                ast::Decl::TsInterface(_) => {
                  continue;
                }
                // export type A = string
                ast::Decl::TsTypeAlias(_) => {
                  continue;
                }
                _ => {
                  is_facade = false;
                  break;
                }
              }
            }
            // e.g. export * from 'b'
            ast::ModuleDecl::ExportNamed(_) => {
              continue;
            }
            // e.g. export default a
            ast::ModuleDecl::ExportDefaultDecl(_) => {
              is_facade = false;
              break;
            }
            // e.g. export default 1
            ast::ModuleDecl::ExportDefaultExpr(_) => {
              is_facade = false;
              break;
            }
            // e.g. export * as a from 'b'
            ast::ModuleDecl::ExportAll(_) => {
              continue;
            }
            // e.g. import TypeScript = TypeScriptServices.TypeScript;
            // not support
            ast::ModuleDecl::TsImportEquals(_) => {
              is_facade = false;
              break;
            }
            // e.g. export = foo
            // not support
            ast::ModuleDecl::TsExportAssignment(_) => {
              is_facade = false;
              break;
            }
            // e.g. export as namespace a
            ast::ModuleDecl::TsNamespaceExport(_) => {
              continue;
            }
          }
        }
        ast::ModuleItem::Stmt(stmt) => {
          if let ast::Stmt::Expr(expr) = stmt {
            if let ast::Expr::Call(call) = expr.expr.as_ref() {
              let is_dynamic_import = call.callee.is_import();
              if is_dynamic_import {
                if call.args.len() == 1 {
                  if let ast::Expr::Lit(lit) = call.args[0].expr.as_ref() {
                    if let ast::Lit::Str(_) = lit {
                      continue;
                    }
                  }
                }
              }
            }
          }
          is_facade = false;
          break;
        }
      }
    }
    self.facade = is_facade;
  }

  fn set_module_syntax(&mut self, value: bool) {
    self.has_module_syntax = value;
  }

  fn detect_syntax(&mut self, module: &mut ast::Module) {
    let mut has_module_syntax = false;
    for item in module.body.iter() {
      // `import` or `export`
      if let ast::ModuleItem::ModuleDecl(_) = item {
        has_module_syntax = true;
        break;
      }
    }
    self.set_module_syntax(has_module_syntax);
  }
}

// visit
impl<'a> VisitMut for ImportExportVisitor<'a> {
  fn visit_mut_module(&mut self, module: &mut ast::Module) {
    self.detect_facade(module);
    self.detect_syntax(module);
    module.visit_mut_children_with(self);
  }

  // normal
  fn visit_mut_module_decl(&mut self, decl: &mut ast::ModuleDecl) {
    match decl {
      // import
      ast::ModuleDecl::Import(import) => {
        self.parse_import(import);
      }
      // export
      // export { a , b as c }
      // export type { a } from 'b'
      // export { a, type b } from 'b'
      // export type * as all from 'b'
      ast::ModuleDecl::ExportNamed(export) => {
        self.parse_named_export(export);
      }
      // export  default   a
      // export default []
      // export default 1
      ast::ModuleDecl::ExportDefaultExpr(export) => {
        self.parse_default_export_expr(export);
      }
      // export namespace A.B {}
      // export class A {}
      // export const a = 1
      // export enum a {}
      // export function a() {}
      // export const a = 1, b = 2
      // export type A = string
      // export interface B {}
      ast::ModuleDecl::ExportDecl(export) => {
        let need_eager_return = self.parse_export_decl(export);
        if need_eager_return {
          // skip visit children
          return;
        }
      }
      // export * from 'vv'
      ast::ModuleDecl::ExportAll(export) => {
        // add import
        let name = export.src.value.to_string();
        let m = self.add_export(ExportSpecifier {
          n: "*".into(),
          ln: Some("".into()),
          src: Some(name),
          wildcard: true,
        });
        if let Some(v) = m.and_then(|f| f.with_ext()) {
          export.src = Box::new(ast::Str::from(v));
        }
      }
      // export default function a () {}
      ast::ModuleDecl::ExportDefaultDecl(export) => {
        self.parse_export_default_decl(export);
      }
      // export = a
      // not support
      ast::ModuleDecl::TsExportAssignment(_) => {}
      // export as namespace a
      ast::ModuleDecl::TsNamespaceExport(_) => {}
      // import TypeScript = TypeScriptServices.TypeScript;
      ast::ModuleDecl::TsImportEquals(_) => {}
    };
    decl.visit_mut_children_with(self)
  }

  // dynamic import or import phase
  fn visit_mut_expr(&mut self, node: &mut ast::Expr) {
    if let ast::Expr::Call(call) = node {
      if let ast::Callee::Import(import) = call.callee {
        let first_arg = call.args.get(0);
        if let Some(arg) = first_arg {
          let mut name = None;

          // get static value
          match arg.expr.as_ref() {
            // import('abc')
            ast::Expr::Lit(lit) => {
              if let ast::Lit::Str(src) = lit {
                name = Some(src.value.to_string());
              }
            }
            // import(`abc`)
            ast::Expr::Tpl(_tpl) => {
              // TODO: actually, we know what is in there. but `es-module-lexer` does not know.
            }
            _ => {}
          }

          // calc assert
          let second_arg = call.args.get(1);
          if let Some(arg) = second_arg {
            // support object only
            if let ast::Expr::Object(obj) = arg.expr.as_ref() {}
          }

          let mut t: Option<ImportType> = None;

          match import.phase {
            ImportPhase::Defer => {
              // import.defer('...')
              // https://github.com/swc-project/swc/blob/a9bab833ba6370a66ab8d7ac209d89ad2ea4c005/crates/swc_ecma_parser/src/parser/expr.rs#L2084
              // do nothing
            }
            ImportPhase::Evaluation => {
              // import('...')
              t = Some(ImportType::Dynamic);
            }
            ImportPhase::Source => {
              // import.source('...')
              t = Some(ImportType::DynamicSourcePhase);
            }
          }

          if t.is_some() {
            self.add_import(ImportSpecifier {
              src: name,
              t: t.unwrap(),
            });
          }
        }
      }
    }
    node.visit_mut_children_with(self);
  }

  fn visit_mut_import_phase(&mut self, phase: &mut ImportPhase) {
    match phase {
      ImportPhase::Defer => {}
      ImportPhase::Evaluation => {}
      ImportPhase::Source => {
        // FIXME: maybe we should set has_module_syntax to true
      }
    }
    phase.visit_mut_children_with(self);
  }

  // import.meta.xxx
  // import.meta
  fn visit_mut_meta_prop_expr(&mut self, meta: &mut ast::MetaPropExpr) {
    self.add_import(ImportSpecifier {
      src: None,
      t: ImportType::ImportMeta,
    });
    // `import.meta` can only appear in module
    self.set_module_syntax(true);
    meta.visit_mut_children_with(self);
  }
}
