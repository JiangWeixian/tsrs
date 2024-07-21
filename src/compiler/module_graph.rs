use std::path::PathBuf;

use sugar_path::SugarPath;

use crate::resolver::Resolver;

#[derive(Default, Clone, Debug)]
pub struct Module {
    pub specifier: String,
    pub context: String,
    pub used: bool,
    pub built_in: bool,
    pub abs_path: String,
    pub relative_path: String,
}

impl Module {
    // TODO: support custom ext
    pub fn with_ext(&self) -> Option<String> {
        let path = self
            .relative_path
            .as_path()
            .with_extension("")
            .with_extension("js");
        path.to_str().map(|f| f.to_string())
    }
}

#[derive(Default, Debug)]
pub struct ModuleGraph {
    pub modules: Vec<Module>,
    pub resolver: Resolver,
}

impl ModuleGraph {
    pub fn new(resolver: Resolver) -> ModuleGraph {
        Self {
            modules: Default::default(),
            resolver,
        }
    }
    pub fn add_module(&mut self, module: Module) {
        self.modules.push(module);
    }
    pub fn resolve_module(&mut self, specifier: Option<String>, context: String) -> Option<Module> {
        if let Some(sp) = specifier {
            let module = match self.resolver.resolve(&sp, &context) {
                Some(resolved) => {
                    let m = Module {
                        specifier: sp,
                        context,
                        abs_path: resolved.abs_path.unwrap_or_default(),
                        relative_path: resolved.relative_path.unwrap_or_default(),
                        ..Default::default()
                    };
                    self.add_module(m.clone());
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
        self.modules.iter_mut().filter(|module| !module.used)
    }
    pub fn get_unused_modules_size(&self) -> usize {
        let modules: Vec<Module> = self
            .modules
            .clone()
            .into_iter()
            .filter(|f| !f.used)
            .collect();
        modules.len()
    }
}
