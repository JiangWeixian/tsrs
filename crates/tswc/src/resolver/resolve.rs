use std::path::PathBuf;

use log::debug;
use oxc_resolver::{
  ResolveError, ResolveOptions, Resolver as OxcResolver, TsconfigOptions, TsconfigReferences,
};
use sugar_path::SugarPath;

#[derive(Default, Debug)]
pub struct Resolver {
  cjs_resolver: OxcResolver,
  mjs_resolver: OxcResolver,
  options: ResolverOptions,
}

fn find_up_dir(context: PathBuf) -> Option<String> {
  if context.is_dir() {
    context.to_str().map(|f| f.to_string())
  } else {
    context
      .parent()
      .and_then(|parent| find_up_dir(parent.to_path_buf()))
  }
}

pub struct ResolvedSpecifier {
  pub abs_path: Option<String>,
  // TODO: remove this one
  pub relative_path: Option<String>,
  pub context: Option<String>,
  pub is_node_modules: bool,
  pub built_in: bool,
  pub not_found: bool,
}

#[derive(Debug)]
pub struct ResolverOptions {
  pub externals: Vec<String>,
  pub modules: Vec<String>,
  pub tsconfig: PathBuf,
}

impl Default for ResolverOptions {
  fn default() -> Self {
    ResolverOptions {
      externals: vec![],
      modules: vec!["node_modules".into()],
      tsconfig: PathBuf::default(),
    }
  }
}

impl Resolver {
  pub fn new(options: ResolverOptions) -> Resolver {
    let resolver_options = ResolverOptions {
      externals: options.externals,
      tsconfig: options.tsconfig.clone(),
      modules: options.modules.clone(),
      ..ResolverOptions::default()
    };
    let cjs_resolved_options = ResolveOptions {
      tsconfig: Some(TsconfigOptions {
        config_file: options.tsconfig.clone(),
        references: TsconfigReferences::Auto,
      }),
      // TODO: exts should config
      extensions: vec![
        ".ts".into(),
        ".tsx".into(),
        ".js".into(),
        ".jsx".into(),
        ".json".into(),
      ],
      exports_fields: vec![vec!["exports".into()]],
      // TODO: create esm resolver
      // TODO: create browser resolver
      condition_names: vec!["node".into(), "require".into(), "import".into()],
      builtin_modules: true,
      main_fields: vec!["main".into()],
      symlinks: true,
      modules: options.modules.clone(),
      ..ResolveOptions::default()
    };
    let cjs_resolver = OxcResolver::new(cjs_resolved_options);
    let mjs_resolved_options = ResolveOptions {
      tsconfig: Some(TsconfigOptions {
        config_file: options.tsconfig,
        references: TsconfigReferences::Auto,
      }),
      // TODO: exts should config
      extensions: vec![
        ".ts".into(),
        ".tsx".into(),
        ".js".into(),
        ".jsx".into(),
        ".json".into(),
      ],
      exports_fields: vec![vec!["exports".into()]],
      // TODO: create esm resolver
      // TODO: create browser resolver
      condition_names: vec!["node".into(), "import".into(), "require".into()],
      builtin_modules: true,
      main_fields: vec!["module".into()],
      symlinks: true,
      modules: options.modules,
      ..ResolveOptions::default()
    };
    let mjs_resolver = OxcResolver::new(mjs_resolved_options);
    Self {
      cjs_resolver,
      mjs_resolver,
      options: resolver_options,
    }
  }
  pub fn is_node_modules(&self, file_path: &Option<String>) -> bool {
    match file_path {
      None => false,
      Some(path) => path.contains("node_modules"),
    }
  }
  pub fn resolve(&self, specifier: &str, context: &str) -> Option<ResolvedSpecifier> {
    let path_str = find_up_dir(PathBuf::from(context));

    if path_str.is_none() {
      return None;
    }

    let path = path_str.clone().unwrap();
    let path = path.as_path();

    if self.options.externals.contains(&String::from(specifier)) {
      return Some(ResolvedSpecifier {
        is_node_modules: true,
        abs_path: None,
        relative_path: None,
        context: path_str,
        built_in: false,
        not_found: false,
      });
    };

    debug!(target: "tswc", "resolve {:?} {:?}", specifier, path_str);

    assert!(
      path.is_dir(),
      "{path:?} must be a directory that will be resolved against."
    );
    assert!(path.is_absolute(), "{path:?} must be an absolute path.",);

    let resolver = &self.cjs_resolver;
    let mut built_in = false;

    let resolved_path: Option<String> = match resolver.resolve(path, &specifier) {
      Err(error) => {
        match error {
          ResolveError::Builtin(spec) => {
            built_in = true;
            return Some(ResolvedSpecifier {
              abs_path: Some(spec),
              relative_path: None,
              context: None,
              built_in,
              is_node_modules: false,
              not_found: false,
            });
          }
          _ => {
            // TODO: should report it?
            debug!(
              target: "tswc",
              "resolve failed from {:?} for {:?}",
              path, &specifier
            );
            return Some(ResolvedSpecifier {
              abs_path: None,
              relative_path: None,
              context: None,
              built_in: false,
              is_node_modules: false,
              not_found: true,
            });
          }
        };
      }
      Ok(resolution) => Some(String::from(resolution.full_path().to_str().unwrap())),
    };
    let relative_path = match &resolved_path {
      Some(resolved_path) => {
        // FIXME: sugar path will remove ./file -> file
        // how to handle this
        if specifier.starts_with('.') {
          Some(specifier.to_string())
        } else {
          let relative_path = resolved_path.as_path().relative(path);
          relative_path.to_str().map(|f| f.to_string())
        }
      }
      None => None,
    };
    Some(ResolvedSpecifier {
      is_node_modules: self.is_node_modules(&resolved_path),
      abs_path: resolved_path,
      relative_path,
      context: path_str,
      built_in,
      not_found: false,
    })
  }
  /// Resolve with esm options
  // TODO: should merge with resolve fn into one, config cjs or mjs resolver by options.module?
  pub fn resolve_module(&self, specifier: &str, context: &str) -> Option<ResolvedSpecifier> {
    let path_str = find_up_dir(PathBuf::from(context));

    if path_str.is_none() {
      return None;
    }

    let path = path_str.clone().unwrap();
    let path = path.as_path();

    if self.options.externals.contains(&String::from(specifier)) {
      return Some(ResolvedSpecifier {
        is_node_modules: true,
        abs_path: None,
        relative_path: None,
        context: path_str,
        built_in: false,
        not_found: false,
      });
    };

    debug!(target: "tswc", "resolve {:?} {:?}", specifier, path_str);

    assert!(
      path.is_dir(),
      "{path:?} must be a directory that will be resolved against."
    );
    assert!(path.is_absolute(), "{path:?} must be an absolute path.",);

    let resolver = &self.mjs_resolver;
    let mut built_in = false;

    let resolved_path: Option<String> = match resolver.resolve(path, &specifier) {
      Err(error) => {
        match error {
          ResolveError::Builtin(spec) => {
            built_in = true;
            return Some(ResolvedSpecifier {
              abs_path: Some(spec),
              relative_path: None,
              context: None,
              built_in,
              is_node_modules: false,
              not_found: false,
            });
          }
          _ => {
            // TODO: should report it?
            debug!(
              target: "tswc",
              "resolve failed from {:?} for {:?}",
              path, &specifier
            );
            return Some(ResolvedSpecifier {
              abs_path: None,
              relative_path: None,
              context: None,
              built_in: false,
              is_node_modules: false,
              not_found: true,
            });
          }
        };
      }
      Ok(resolution) => Some(String::from(resolution.full_path().to_str().unwrap())),
    };
    let relative_path = match &resolved_path {
      Some(resolved_path) => {
        // FIXME: sugar path will remove ./file -> file
        // how to handle this
        if specifier.starts_with('.') {
          Some(specifier.to_string())
        } else {
          let relative_path = resolved_path.as_path().relative(path);
          relative_path.to_str().map(|f| f.to_string())
        }
      }
      None => None,
    };
    Some(ResolvedSpecifier {
      is_node_modules: self.is_node_modules(&resolved_path),
      abs_path: resolved_path,
      relative_path,
      context: path_str,
      built_in,
      not_found: false,
    })
  }
}
