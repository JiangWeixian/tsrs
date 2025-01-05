use log::debug;
use serde::Deserialize;
use swc_core::common::DUMMY_SP;
use swc_core::ecma::ast::*;
use swc_core::ecma::visit::{noop_fold_type, Fold};

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
  pub packages: Vec<String>,
}

#[derive(Debug, Default)]
pub struct NamedImportTransform {
  pub packages: Vec<String>,
}

impl NamedImportTransform {
  pub fn new(config: Config) -> Self {
    NamedImportTransform {
      packages: config.packages,
    }
  }
}

impl Fold for NamedImportTransform {
  noop_fold_type!();

  fn fold_module(&mut self, mut module: Module) -> Module {
    let mut new_items: Vec<ModuleItem> = vec![];
    for item in module.body {
      match item {
        ModuleItem::ModuleDecl(ModuleDecl::Import(decl)) => {
          let src_value = decl.src.value.clone();
          let mut skip_transform = false;
          debug!(
            target: "tswc",
            "self.pages {:?}",
            self.packages
          );
          if self.packages.iter().any(|p| src_value == *p) {
            debug!(
              target: "tswc",
              "self.named_imports src_value {:?}",
              src_value
            );
            for specifier in &decl.specifiers {
              match specifier {
                ImportSpecifier::Named(specifier) => {
                  // Add the import name as string to the set
                  if let Some(imported) = &specifier.imported {
                    match imported {
                      ModuleExportName::Ident(_) => {
                        let specifiers = ImportSpecifier::Named(specifier.clone());
                        let import = ImportDecl {
                          span: DUMMY_SP,
                          src: Box::new(Str {
                            span: DUMMY_SP,
                            value: src_value.clone().into(),
                            raw: None,
                          }),
                          type_only: false,
                          with: None,
                          specifiers: vec![specifiers],
                          phase: Default::default(),
                        };
                        new_items.push(ModuleItem::ModuleDecl(ModuleDecl::Import(import)))
                      }
                      ModuleExportName::Str(_) => {
                        let specifiers = ImportSpecifier::Named(specifier.clone());
                        let import = ImportDecl {
                          span: DUMMY_SP,
                          src: Box::new(Str {
                            span: DUMMY_SP,
                            value: src_value.clone().into(),
                            raw: None,
                          }),
                          type_only: false,
                          with: None,
                          specifiers: vec![specifiers],
                          phase: Default::default(),
                        };
                        new_items.push(ModuleItem::ModuleDecl(ModuleDecl::Import(import)))
                      }
                    }
                  } else {
                    let specifiers = ImportSpecifier::Named(specifier.clone());
                    let import = ImportDecl {
                      span: DUMMY_SP,
                      src: Box::new(Str {
                        span: DUMMY_SP,
                        value: src_value.clone().into(),
                        raw: None,
                      }),
                      type_only: false,
                      with: None,
                      specifiers: vec![specifiers],
                      phase: Default::default(),
                    };
                    new_items.push(ModuleItem::ModuleDecl(ModuleDecl::Import(import)))
                  }
                }
                ImportSpecifier::Default(_) => {
                  skip_transform = true;
                  break;
                }
                ImportSpecifier::Namespace(_) => {
                  skip_transform = true;
                  break;
                }
              }
            }
          } else {
            skip_transform = true
          }
          if skip_transform {
            new_items.push(ModuleItem::ModuleDecl(ModuleDecl::Import(decl)));
          }
        }
        x => {
          new_items.push(x);
        }
      }
    }
    module.body = new_items;
    module
  }
}
