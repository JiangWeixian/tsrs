mod compiler;
mod config;
mod plugins;
mod resolver;
mod utils;

use glob::glob;

use compiler::{compile, Assets, ModuleGraph};
use config::{Config, ConfigOptions};
use resolver::Resolver;
use sugar_path::SugarPath;

fn main() {
    let resolver = Resolver::new();
    let assets = Assets::new();
    let input = String::from(
        "./fixtures/package-a/src"
            .as_path()
            .absolutize()
            .to_str()
            .unwrap(),
    );
    // TODO: ensure input and output
    let output_options = ConfigOptions {
        output: String::from(
            "./fixtures/package-a/dist"
                .as_path()
                .absolutize()
                .to_str()
                .unwrap(),
        ),
        input: input.clone(),
    };
    let config = Config::new(output_options);
    let mut mg = ModuleGraph::new(resolver, config);
    let files = glob("./fixtures/package-a/src/*.ts").expect("Failed to read glob pattern");
    for entry in files {
        match entry {
            Ok(path) => {
                // println!("compile! {:?}", path.to_str().unwrap());
                let resource_path = path.as_path().absolutize();
                mg.resolve_entry_module(Some(
                    resource_path.to_str().unwrap_or_default().to_string(),
                ));
                let resource_path_str = resource_path.to_str().unwrap();
                // let result = compile(resource_path_str, &mut mg);
                // output.output(resource_path_str, result);
            }
            Err(e) => println!("{:?}", e),
        }
    }
    while mg.get_unused_modules_size() != 0 {
        let paths_to_compile: Vec<_> = mg
            .get_unused_modules()
            .map(|decl| {
                decl.used = true;
                println!("compile! {:?} {:?}", &decl.abs_path, &decl.v_abs_path);
                (decl.abs_path.clone(), decl.v_abs_path.clone())
            })
            .collect();
        // Assuming `module_graph::Module` implements `Debug`
        // for module in mg.get_unused_modules() {
        //     println!("used {:?}", module.used);
        // }
        for (resolved_path, output_path) in paths_to_compile {
            let resource_path = resolved_path.as_path().absolutize();
            let resource_path_str = resource_path.to_str().unwrap();
            let result: swc_core::base::TransformOutput = compile(&resolved_path, &mut mg);
            assets.output(&output_path, result)
        }
    }
}
