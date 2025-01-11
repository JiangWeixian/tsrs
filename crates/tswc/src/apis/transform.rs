use crate::compiler::{compile, optimize, Assets, ModuleGraph, ResolveModuleOptions};
use crate::config::{Config, ConfigOptions};
use crate::resolver::{Format, Resolver, ResolverOptions};
use log::debug;
use napi_derive::napi;
use sugar_path::SugarPath;

pub struct PreOptimizeOptions<'a> {
  pub root: String,
  /// Pre optimized packages
  pub barrel_packages: Vec<String>,
  pub mg: &'a mut ModuleGraph,
}

pub fn pre_optimize(options: PreOptimizeOptions) {
  let PreOptimizeOptions {
    root,
    barrel_packages,
    mut mg,
  } = options;
  for package in barrel_packages {
    mg.resolve_module(ResolveModuleOptions {
      src: Some(package),
      context: root.clone(),
      is_wildcard: Some(true),
      format: Some(Format::ESM),
      ..Default::default()
    });
  }
  while mg.get_wildcard_modules_size() != 0 {
    let paths_to_compile: Vec<_> = {
      let unused_modules = mg.get_wildcard_modules();
      unused_modules
        .map(|decl| {
          decl.optimized = true;
          debug!(
              target: "tswc",
              "optimize! {:?}", &decl.abs_path
          );
          (decl.abs_path.clone(), decl.is_script, decl.is_wildcard)
        })
        .collect()
    };
    for (resolved_path, is_script, is_wildcard) in paths_to_compile {
      if is_script {
        optimize(&resolved_path, &mut mg, Some(is_wildcard));
      } else {
      }
    }
  }
  debug!("Finish pre optimized")
}

#[napi(object)]
pub struct TransformOptimizeOptions {
  /// Optimized packages
  pub barrel_packages: Option<Vec<String>>,
}

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
  /// Optimized options
  pub optimize: TransformOptimizeOptions,
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
    optimize,
  } = options;
  let TransformOptimizeOptions { barrel_packages } = optimize;
  let root_cloned = root.clone();
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
    barrel_packages: barrel_packages.clone().unwrap_or_default(),
  };
  let mut config = Config::new(config_options);
  config.resolve_options(&tsconfig_path);
  config.search_files();
  let files = config.files.clone();
  let mut mg = ModuleGraph::new(resolver, config);
  debug!(target: "tswc", "files {:?}", files);
  pre_optimize(PreOptimizeOptions {
    root: root_cloned,
    barrel_packages: barrel_packages.unwrap_or_default(),
    mg: &mut mg,
  });
  for path in files {
    let resource_path = path.as_path().absolutize();
    mg.resolve_entry_module(
      Some(resource_path.to_str().unwrap_or_default().to_string()),
      Some(false),
    );
  }
  while mg.get_unused_modules_size() != 0 {
    // Wrap paths_to_compile with `{}` prevent lifetime issue
    let paths_to_compile: Vec<_> = {
      let unused_modules = mg.get_unused_modules();
      unused_modules
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
        .collect()
    };
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
