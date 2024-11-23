use std::fs;
use std::path::Path;
use std::sync::Arc;

use super::{ModuleGraph, SwcCompiler};
use crate::plugins::ImportExportVisitor;
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
  let export_import_visitor = ImportExportVisitor::new(module_graph, context);
  let ch = chain!(
    // as_folder(DemoVisitor::default()),
    as_folder(export_import_visitor),
    noop()
  );
  ch
}

#[allow(clippy::too_many_arguments)]
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
