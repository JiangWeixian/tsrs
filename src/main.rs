mod compiler;
mod plugins;
mod resolver;
mod utils;
use glob::glob;

use compiler::{compile, ModuleGraph};
use resolver::Resolver;

fn main() {
    let resolver = Resolver::new();
    let mut mg = ModuleGraph::new(resolver);
    let files = glob("./fixtures/package-a/src/*.ts").expect("Failed to read glob pattern");
    for entry in files {
        match entry {
            Ok(path) => {
                println!("compile! {:?}", path.to_str().unwrap());
                compile(path.to_str().unwrap(), &mut mg);
            }
            Err(e) => println!("{:?}", e),
        }
    }
    while mg.get_unused_modules_size() != 0 {
        let paths_to_compile: Vec<_> = mg
            .get_unused_modules()
            .map(|decl| {
                decl.used = true;
                decl.abs_path.clone()
            })
            .collect();
        // Assuming `module_graph::Module` implements `Debug`
        // for module in mg.get_unused_modules() {
        //     println!("used {:?}", module.used);
        // }
        for resolved_path in paths_to_compile {
            println!("compile! {:?}", &resolved_path);
            compile(&resolved_path, &mut mg);
        }
    }
}
