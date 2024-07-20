use std::path::PathBuf;

use oxc_resolver::{ResolveError, ResolveOptions, Resolver as OxcResolver};

pub struct Resolver {
    resolver: OxcResolver,
}

impl Resolver {
    pub fn new() -> Resolver {
        let resolved_options = ResolveOptions {
            extensions: vec![".js".into(), ".ts".into()],
            extension_alias: vec![(".js".into(), vec![".ts".into()])],
            builtin_modules: true,
            ..ResolveOptions::default()
        };
        let resolver = OxcResolver::new(resolved_options);
        Self { resolver }
    }
    pub fn resolve(&self, context: String, specifier: String) -> Option<String> {
        let path = PathBuf::from(context);

        assert!(
            path.is_dir(),
            "{path:?} must be a directory that will be resolved against."
        );
        assert!(path.is_absolute(), "{path:?} must be an absolute path.",);

        // println!("path: {path:?}");
        // println!("specifier: {specifier}");

        let resolver = &self.resolver;

        let resolved_path: Option<String> = match resolver.resolve(path, &specifier) {
            Err(error) => {
                let resolved: Option<String> = match error {
                    ResolveError::Builtin(spec) => Some(spec.to_string()),
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
        resolved_path
    }
}
