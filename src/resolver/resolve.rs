use std::path::PathBuf;

use oxc_resolver::{ResolveError, ResolveOptions, Resolver as OxcResolver};
use sugar_path::SugarPath;

#[derive(Default, Debug)]
pub struct Resolver {
    resolver: OxcResolver,
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
}

impl Resolver {
    pub fn new() -> Resolver {
        let resolved_options = ResolveOptions {
            extensions: vec![".js".into(), ".ts".into()],
            builtin_modules: true,
            ..ResolveOptions::default()
        };
        let resolver = OxcResolver::new(resolved_options);
        Self { resolver }
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

        println!("resolve {:?} {:?}", specifier, path_str);

        assert!(
            path.is_dir(),
            "{path:?} must be a directory that will be resolved against."
        );
        assert!(path.is_absolute(), "{path:?} must be an absolute path.",);

        let resolver = &self.resolver;
        let mut built_in = false;

        let resolved_path: Option<String> = match resolver.resolve(path, &specifier) {
            Err(error) => {
                let resolved: Option<String> = match error {
                    ResolveError::Builtin(spec) => {
                        built_in = true;
                        return Some(ResolvedSpecifier {
                            abs_path: Some(spec),
                            relative_path: None,
                            context: None,
                            built_in,
                            is_node_modules: false,
                        });
                    }
                    _ => {
                        println!("Error: {error}");
                        None
                    }
                };
                resolved
            }
            Ok(resolution) => {
                // println!("Resolved: {:?}", resolution.full_path());
                Some(String::from(resolution.full_path().to_str().unwrap()))
            }
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
        })
    }
}
