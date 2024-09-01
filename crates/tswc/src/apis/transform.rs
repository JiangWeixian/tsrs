use crate::compiler::{compile, Assets, ModuleGraph};
use crate::config::{Config, ConfigOptions};
use crate::resolver::Resolver;
use log::debug;
use napi_derive::napi;
use sugar_path::SugarPath;

#[napi(object)]
pub struct TransformOptions {
  pub root: String,
}

pub fn transform(options: TransformOptions) {
  env_logger::init();
  let resolver = Resolver::new();
  let assets = Assets::new();
  let TransformOptions { root } = options;
  // TODO: read from args.env
  let root = root.as_path().absolutize();
  debug!(target: "tswc", "root {:?}", root);
  let input = root.join("src");
  let output = root.join("dist");
  let tsconfig_path = root.join("tsconfig.json");
  let config_options = ConfigOptions {
    root,
    input,
    output,
  };
  let mut config = Config::new(config_options);
  config.resolve_options(&tsconfig_path);
  config.search_files();
  let inputs = config.inputs.clone();
  let mut mg = ModuleGraph::new(resolver, config);
  debug!(target: "tswc", "inputs {:?}", inputs);
  for path in inputs {
    let resource_path = path.as_path().absolutize();
    mg.resolve_entry_module(Some(resource_path.to_str().unwrap_or_default().to_string()));
  }
  while mg.get_unused_modules_size() != 0 {
    let paths_to_compile: Vec<_> = mg
      .get_unused_modules()
      .map(|decl| {
        decl.used = true;
        debug!(
            target: "tswc",
            "compile! {:?} {:?}", &decl.abs_path, &decl.v_abs_path
        );
        (decl.abs_path.clone(), decl.v_abs_path.clone())
      })
      .collect();
    for (resolved_path, output_path) in paths_to_compile {
      debug!(target: "tswc", "output {}", output_path);
      let result = compile(&resolved_path, &mut mg);
      assets.output(&output_path, result)
    }
  }
}
