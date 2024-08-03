use std::collections::HashMap;
use std::path::{Path, PathBuf};

use sugar_path::SugarPath;

use crate::config::Config;
use crate::resolver::Resolver;

fn common_path_prefix(p1: &Path, p2: &Path) -> PathBuf {
    let mut common_prefix = PathBuf::new();
    let mut iter1 = p1.components();
    let mut iter2 = p2.components();

    while let (Some(c1), Some(c2)) = (iter1.next(), iter2.next()) {
        if c1 == c2 {
            common_prefix.push(c1.as_os_str());
        } else {
            break;
        }
    }

    common_prefix
}

fn replace_common_prefix(p1: &Path, p2: &Path, new_prefix: &Path) -> String {
    let common_prefix = common_path_prefix(p1, p2);
    let new_p1 = p1
        .strip_prefix(&common_prefix)
        .map(|suffix| new_prefix.join(suffix))
        .unwrap_or_else(|_| p1.to_path_buf());
    String::from(new_p1.to_str().unwrap_or_default())
}

#[derive(Default, Clone, Debug)]
pub struct Module {
    /// The imported named of the module
    pub specifier: String,
    /// The dir of current importee file
    pub context: String,
    /// is current module is compiled
    pub used: bool,
    /// Builtin node native modules
    pub built_in: bool,
    /// imported from node_modules
    pub is_node_modules: bool,
    /// input files from options.include
    pub is_entry: bool,
    /// Resolved absolute filepath of specifier
    pub abs_path: String,
    /// Relative path relative to abs_path
    pub relative_path: String,
    /// Virtual absolute filepath, rewrite abs_path based on output.dir
    pub v_abs_path: String,
    /// Relative path relative to v_abs_path
    pub v_relative_path: String,
}

impl Module {
    // TODO: support custom ext
    pub fn with_ext(&self) -> Option<String> {
        if self.built_in || self.is_node_modules {
            return Some(self.specifier.clone());
        }
        let path = self
            .v_relative_path
            .as_path()
            .with_extension("")
            .with_extension("js");
        path.to_str().map(|f| f.to_string())
    }
}

#[derive(Default, Debug)]
pub struct ModuleGraph {
    pub modules: HashMap<String, Module>,
    pub resolver: Resolver,
    pub config: Config,
}

impl ModuleGraph {
    pub fn new(resolver: Resolver, config: Config) -> ModuleGraph {
        Self {
            modules: Default::default(),
            resolver,
            config,
        }
    }
    pub fn add_module(&mut self, abs_path: String, module: Module) {
        if !self.modules.contains_key(&abs_path) {
            self.modules.insert(abs_path, module);
        }
    }
    pub fn resolve_entry_module(&mut self, specifier: Option<String>) -> Option<Module> {
        if let Some(sp) = specifier {
            let abs_path = {
                let path = sp.as_path().absolutize();
                path.to_str().unwrap_or_default().to_string()
            };
            let v_abs_path = abs_path.replace(
                &self.config.resolved_options.input,
                &self.config.resolved_options.output.to_str().unwrap(),
            );
            let m = Module {
                specifier: sp,
                v_abs_path: String::from(v_abs_path),
                abs_path: String::from(abs_path.clone()),
                is_entry: true,
                ..Default::default()
            };
            self.add_module(abs_path, m.clone());
            Some(m)
        } else {
            None
        }
    }
    pub fn resolve_module(&mut self, specifier: Option<String>, context: String) -> Option<Module> {
        if let Some(sp) = specifier {
            let module = match self.resolver.resolve(&sp, &context) {
                Some(resolved) => {
                    let abs_path = resolved.abs_path;
                    let v_abs_path = abs_path.as_ref().and_then(|f| {
                        Some(replace_common_prefix(
                            f.as_path(),
                            &self.config.resolved_options.input.as_path(),
                            &self.config.resolved_options.output.as_path(),
                        ))
                    });
                    let relative_path = resolved.relative_path;
                    let context = resolved.context;
                    let v_context = context.clone().and_then(|f| {
                        Some(replace_common_prefix(
                            &f.as_path(),
                            &self.config.resolved_options.input.as_path(),
                            &self.config.resolved_options.output.as_path(),
                        ))
                    });
                    let v_relative_path = v_abs_path.as_ref().and_then(|f| {
                        let relative_path = f
                            .as_path()
                            .relative(v_context.clone().unwrap_or_default().as_path());
                        relative_path.to_str().map(|f| {
                            if f.starts_with(".") {
                                f.to_string()
                            } else {
                                format!("./{}", f)
                            }
                        })
                    });
                    println!(
                        "relative_path {:?} v_relative_path {:?} abs_path {:?} v_abs_path {:?} context {:?}",
                        relative_path, v_relative_path, abs_path, v_abs_path, v_context
                    );
                    let m = Module {
                        specifier: sp,
                        context: context.unwrap_or_default(),
                        abs_path: abs_path.clone().unwrap_or_default(),
                        v_abs_path: v_abs_path.unwrap_or_default(),
                        relative_path: relative_path.unwrap_or_default(),
                        v_relative_path: v_relative_path.unwrap_or_default(),
                        used: resolved.built_in || resolved.is_node_modules,
                        is_node_modules: resolved.is_node_modules,
                        built_in: resolved.built_in,
                        ..Default::default()
                    };
                    // TODO: fix unwrap
                    self.add_module(abs_path.unwrap_or_default(), m.clone());
                    Some(m)
                }
                None => None,
            };
            module
        } else {
            None
        }
    }
    pub fn get_unused_modules(&mut self) -> impl Iterator<Item = &mut Module> {
        self.modules.values_mut().filter(|module| !module.used)
    }
    pub fn get_unused_modules_size(&self) -> usize {
        let modules: Vec<&Module> = self
            .modules
            .values()
            .into_iter()
            .filter(|f| !f.used)
            .collect();
        modules.len()
    }
}
