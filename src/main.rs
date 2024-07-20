mod compiler;
mod plugins;
mod resolver;
mod utils;
use glob::glob;
use std::path::Path;

use compiler::{compile, ModuleGraph};
use resolver::Resolver;

fn main() {
    let mut mg = ModuleGraph::default();
    let files = glob("./fixtures/package-a/src/*.ts").expect("Failed to read glob pattern");
    for entry in files {
        match entry {
            Ok(path) => {
                compile(path.to_str().unwrap(), &mut mg);
            }
            Err(e) => println!("{:?}", e),
        }
    }
    let resolver = Resolver::new();
    while mg.get_unused_modules_size() != 0 {
        let paths_to_compile: Vec<_> = mg
            .get_unused_modules()
            .map(|decl| {
                let resolved_path = resolver.resolve(
                    Path::new(&decl.context)
                        .parent()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .into(),
                    decl.specifier.clone(),
                );
                decl.set_file_path(&resolved_path.clone().unwrap());
                decl.used = true;
                resolved_path
            })
            .collect();
        // Assuming `module_graph::Module` implements `Debug`
        // for module in mg.get_unused_modules() {
        //     println!("used {:?}", module.used);
        // }
        for resolved_path in paths_to_compile {
            println!("compile! {:?}", &resolved_path);
            compile(&resolved_path.unwrap(), &mut mg);
        }
    }
}
