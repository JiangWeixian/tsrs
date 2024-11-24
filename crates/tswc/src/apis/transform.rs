use crate::compiler::{compile, optimize, Assets, ModuleGraph};
use crate::config::{Config, ConfigOptions};
use crate::resolver::{Resolver, ResolverOptions};
use log::debug;
use napi_derive::napi;
use sugar_path::SugarPath;

#[napi(object)]
pub struct TransformOptions {
  pub root: String,
  // override tsconfig outDir
  pub output: Option<String>,
  pub externals: Option<Vec<String>>,
  // override tsconfig exclude
  pub exclude: Option<Vec<String>>,
  // TODO: should nested in resolve config
  pub modules: Option<Vec<String>>,
}

pub fn transform(options: TransformOptions) {
  env_logger::init();
  let assets = Assets::new();
  let TransformOptions {
    root,
    output,
    externals,
    exclude,
    modules,
  } = options;
  let root = root.as_path().absolutize();
  let tsconfig_path = root.join("tsconfig.json");
  let resolver = Resolver::new(ResolverOptions {
    externals: externals.unwrap_or(vec![]),
    modules: modules.unwrap_or(vec!["node_modules".into()]),
    tsconfig: tsconfig_path.clone(),
  });
  debug!(target: "tswc", "root {:?}", root);
  let config_options = ConfigOptions {
    root,
    output,
    exclude,
  };
  let mut config = Config::new(config_options);
  config.resolve_options(&tsconfig_path);
  config.search_files();
  let files = config.files.clone();
  let mut mg = ModuleGraph::new(resolver, config);
  debug!(target: "tswc", "files {:?}", files);
  for path in files {
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
        (
          decl.abs_path.clone(),
          decl.v_abs_path.clone(),
          decl.is_script,
        )
      })
      .collect();
    for (resolved_path, output_path, is_script) in paths_to_compile {
      debug!(target: "tswc", "output {} {}", output_path, is_script);
      if is_script {
        let result = compile(&resolved_path, &mut mg);
        assets.output(&output_path, result)
      } else {
        assets.copy(&output_path, &resolved_path)
      }
    }
  }
}

#[napi(object)]
pub struct PreOptimizeOptions {
  pub root: String,
  // override tsconfig outDir
  pub output: Option<String>,
  // override tsconfig outDir
  pub packages: Vec<String>,
  pub externals: Option<Vec<String>>,
  // override tsconfig exclude
  pub exclude: Option<Vec<String>>,
  // TODO: should nested in resolve config
  pub modules: Option<Vec<String>>,
}

pub fn pre_optimize(options: PreOptimizeOptions) {
    env_logger::init();
    let assets = Assets::new();
    let PreOptimizeOptions {
      root,
      externals,
      exclude,
      modules,
      output,
      packages
    } = options;
    let root = root.as_path().absolutize();
    let tsconfig_path = root.join("tsconfig.json");
    let resolver = Resolver::new(ResolverOptions {
      externals: externals.unwrap_or(vec![]),
      modules: modules.unwrap_or(vec!["node_modules".into()]),
      tsconfig: tsconfig_path.clone(),
    });
    debug!(target: "tswc", "root {:?}", root);
    let config_options = ConfigOptions {
      root: root.clone(),
      output,
      exclude,
    };
    let mut config = Config::new(config_options);
    config.resolve_options(&tsconfig_path);
    let files = config.files.clone();
    let mut mg = ModuleGraph::new(resolver, config);
    for package in packages {
      let resolved_package_entry = mg.resolver.resolve_module(&package, root.to_str().unwrap());
      if let Some(resolved_package_entry_path) = resolved_package_entry {
          println!("resolved_package_entry_path.abs_path: {:?}", resolved_package_entry_path.abs_path);
          mg.resolve_entry_module(resolved_package_entry_path.abs_path);
      }
    }
    while mg.get_unused_modules_size() != 0 {
      let paths_to_compile: Vec<_> = mg
        .get_unused_modules()
        .map(|decl| {
          decl.used = true;
          debug!(
              target: "tswc",
              "compile! {:?}", &decl.abs_path
          );
          (
            decl.abs_path.clone(),
            decl.is_script,
          )
        })
        .collect();
      for (resolved_path, is_script) in paths_to_compile {
        if is_script {
          let result = optimize(&resolved_path, &mut mg);
          println!("exports_map {:?}", mg.modules.get(&resolved_path));
        } else {
        }
      }
    }
}
