#[derive(Default, Clone, Debug)]
pub struct Module {
    pub specifier: String,
    pub context: String,
    pub used: bool,
    pub file_path: String,
}

impl Module {
    pub fn set_file_path(&mut self, file_path: &str) {
        self.file_path = file_path.into();
    }
}

#[derive(Default, Debug, Clone)]
pub struct ModuleGraph {
    pub modules: Vec<Module>,
}

impl ModuleGraph {
    fn new() -> ModuleGraph {
        Self {
            modules: Default::default(),
        }
    }
    pub fn add_module(&mut self, specifier: Option<String>, context: String) {
        self.modules.push(Module {
            specifier: specifier.unwrap_or_default(),
            context: context.into(),
            ..Default::default()
        })
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
