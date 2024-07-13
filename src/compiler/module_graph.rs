#[derive(Default, Clone)]
pub struct Module {
    pub specifier: String,
    pub context: String,
    pub used: bool,
    pub real_path: String,
}

#[derive(Default)]
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
}
