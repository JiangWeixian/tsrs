mod compiler;
mod plugins;
mod resolver;
mod utils;
use std::path::Path;

use compiler::{compile, ModuleGraph};
use resolver::resolve;

fn main() {
    let mut mg = ModuleGraph::default();
    compile("./fixtures/index.ts", &mut mg);
    let paths_to_compile: Vec<_> = mg
        .modules
        .iter()
        .map(|decl| {
            resolve(
                Path::new(&decl.context)
                    .parent()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .into(),
                decl.specifier.clone(),
            )
        })
        .collect();
    for resolved_path in paths_to_compile {
        println!("Hello, world! {:?}", &resolved_path);
        compile(&resolved_path.unwrap(), &mut mg);
    }
}
