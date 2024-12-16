use std::fs;
use std::path::Path;
use std::sync::Arc;

use super::{ModuleGraph, SwcCompiler};
use crate::plugins::{
  Barrel, BarrelConfig, ImportExportVisitor, NamedImportTransform, NamedImportTransformConfig,
};
use swc_core::base::config::{Config, JsMinifyFormatOptions, JscConfig, ModuleConfig, Options};
use swc_core::base::TransformOutput;
use swc_core::common::{chain, comments::Comments, Mark, SourceMap};
use swc_core::ecma::{
  ast::EsVersion,
  transforms::base::pass::noop,
  visit::{as_folder, Fold},
};
use tsconfig::TsConfig;

#[allow(clippy::too_many_arguments)]
pub fn transform<'a>(
  _resource_path: &'a Path,
  _comments: Option<&'a dyn Comments>,
  _top_level_mark: Mark,
  _unresolved_mark: Mark,
  _cm: Arc<SourceMap>,
  module_graph: &'a mut ModuleGraph,
  context: String,
) -> impl Fold + 'a {
  let packages = module_graph.config.resolved_options.barrel_packages.clone();
  let export_import_visitor = ImportExportVisitor::new(module_graph, context);
  let named_import_transform_visitor =
    NamedImportTransform::new(NamedImportTransformConfig { packages });
  let ch = chain!(
    named_import_transform_visitor,
    as_folder(export_import_visitor),
    noop()
  );
  ch
}

pub trait IntoOptions {
  fn into_options(self) -> Options;
}

impl IntoOptions for TsConfig {
  fn into_options(self) -> Options {
    let target = self
      .compiler_options
      .as_ref()
      .and_then(|f| match &f.target {
        Some(target) => {
          let target = match target {
            tsconfig::Target::Es3 => EsVersion::Es3,
            tsconfig::Target::Es5 => EsVersion::Es5,
            tsconfig::Target::Es6 => EsVersion::Es2015,
            tsconfig::Target::Es7 => EsVersion::Es2016,
            tsconfig::Target::Es2015 => EsVersion::Es2015,
            tsconfig::Target::Es2016 => EsVersion::Es2016,
            tsconfig::Target::Es2017 => EsVersion::Es2017,
            tsconfig::Target::Es2018 => EsVersion::Es2018,
            tsconfig::Target::Es2019 => EsVersion::Es2019,
            tsconfig::Target::Es2020 => EsVersion::Es2020,
            tsconfig::Target::EsNext => EsVersion::EsNext,
            tsconfig::Target::Other(target) => match target.as_str() {
              "ES2021" => EsVersion::Es2021,
              "ES2022" => EsVersion::Es2022,
              _ => EsVersion::Es3,
            },
          };
          Some(target)
        }
        None => Some(EsVersion::Es3),
      })
      .unwrap_or(EsVersion::Es3);
    let module = self
      .compiler_options
      .as_ref()
      .and_then(|f| match &f.module {
        Some(module) => {
          let module = match module {
            tsconfig::Module::CommonJs => ModuleConfig::CommonJs(Default::default()),
            tsconfig::Module::Amd => ModuleConfig::Amd(Default::default()),
            tsconfig::Module::Umd => ModuleConfig::Umd(Default::default()),
            tsconfig::Module::System => ModuleConfig::SystemJs(Default::default()),
            tsconfig::Module::Es6 => ModuleConfig::Es6(Default::default()),
            tsconfig::Module::EsNext => ModuleConfig::Es6(Default::default()),
            tsconfig::Module::Es2015 => ModuleConfig::Es6(Default::default()),
            tsconfig::Module::Es2020 => ModuleConfig::Es6(Default::default()),
            tsconfig::Module::Other(module) => {
              if module == "Node16" || module == "NodeNext" {
                ModuleConfig::NodeNext(Default::default())
              } else {
                let m = if target == EsVersion::Es3 || target == EsVersion::Es5 {
                  ModuleConfig::CommonJs(Default::default())
                } else {
                  ModuleConfig::Es6(Default::default())
                };
                m
              }
            }
            tsconfig::Module::None => {
              let m = if target == EsVersion::Es3 || target == EsVersion::Es5 {
                ModuleConfig::CommonJs(Default::default())
              } else {
                ModuleConfig::Es6(Default::default())
              };
              m
            }
          };
          Some(module)
        }
        None => Some(ModuleConfig::CommonJs(Default::default())),
      });

    Options {
      config: Config {
        module,
        jsc: JscConfig {
          target: Some(target),
          ..Default::default()
        },
        ..Default::default()
      },
      ..Default::default()
    }
  }
}

#[allow(clippy::too_many_arguments)]
pub fn compile<'a>(resource_path: &str, mut module_graph: &'a mut ModuleGraph) -> TransformOutput {
  let options = module_graph.config.tsconfig.clone().unwrap().into_options();
  // to absolute path
  let resource_path = Path::new(resource_path).canonicalize().expect("TODO:");
  let source = fs::read_to_string(&resource_path).expect("failed to read file");
  let c = SwcCompiler::new(resource_path.to_path_buf(), source.clone(), options)
    .map_err(|err| anyhow::Error::from(err))
    .expect("TODO:");
  let options = c.options();
  let top_level_mark = options
    .top_level_mark
    .expect("`top_level_mark` should be initialized");
  let unresolved_mark = options
    .unresolved_mark
    .expect("`unresolved_mark` should be initialized");

  let built = c
    .parse(None, |_| {
      transform(
        &resource_path,
        Some(c.comments()),
        top_level_mark,
        unresolved_mark,
        c.cm().clone(),
        &mut module_graph,
        resource_path.to_str().unwrap().to_string(),
      )
    })
    .expect("TODO:");
  let program = c
    .transform(built)
    .map_err(|err| anyhow::Error::from(err))
    .expect("TODO:");
  let format_opt = JsMinifyFormatOptions {
    ..Default::default()
  };
  let output = c.print(
    &program,
    c.cm().clone(),
    EsVersion::Es2022,
    super::compiler::SourceMapConfig::default(),
    None,
    false,
    None,
    &format_opt,
  );
  output.unwrap()
}

#[allow(clippy::too_many_arguments)]
pub fn transform_with_optimize<'a>(
  _resource_path: &'a Path,
  _comments: Option<&'a dyn Comments>,
  _top_level_mark: Mark,
  _unresolved_mark: Mark,
  _cm: Arc<SourceMap>,
  module_graph: &'a mut ModuleGraph,
  context: String,
) -> impl Fold + 'a {
  let barrel = Barrel::new(module_graph, context, BarrelConfig { wildcard: false });
  let ch = chain!(barrel, noop());
  ch
}

// Do some optimization.
// Job: Barrel optimize
pub fn optimize<'a>(resource_path: &str, mut module_graph: &'a mut ModuleGraph) -> TransformOutput {
  let options = module_graph.config.tsconfig.clone().unwrap().into_options();
  // to absolute path
  let resource_path = Path::new(resource_path).canonicalize().expect("TODO:");
  let source = fs::read_to_string(&resource_path).expect("failed to read file");
  let c = SwcCompiler::new(resource_path.to_path_buf(), source.clone(), options)
    .map_err(|err| anyhow::Error::from(err))
    .expect("TODO:");
  let options = c.options();
  let top_level_mark = options
    .top_level_mark
    .expect("`top_level_mark` should be initialized");
  let unresolved_mark = options
    .unresolved_mark
    .expect("`unresolved_mark` should be initialized");

  let built = c
    .parse(None, |_| {
      transform_with_optimize(
        &resource_path,
        Some(c.comments()),
        top_level_mark,
        unresolved_mark,
        c.cm().clone(),
        &mut module_graph,
        resource_path.to_str().unwrap().to_string(),
      )
    })
    .expect("TODO:");
  let program = c
    .transform(built)
    .map_err(|err| anyhow::Error::from(err))
    .expect("TODO:");

  let format_opt = JsMinifyFormatOptions {
    ..Default::default()
  };
  let output = c.print(
    &program,
    c.cm().clone(),
    EsVersion::Es2022,
    super::compiler::SourceMapConfig::default(),
    None,
    false,
    None,
    &format_opt,
  );
  output.unwrap()
}
