mod compiler;
mod config;
mod plugins;
mod resolver;
mod utils;

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
    let config_options = ConfigOptions {
        root: "./fixtures/package-a/".as_path().absolutize(),
        input: input.clone(),
        ..Default::default()
    };
    let mut config = Config::new(config_options);
    config.resolve_options(
        "./fixtures/package-a/tsconfig.json"
            .as_path()
            .absolutize()
            .as_path(),
    );
    config.search_files();
    let inputs = config.inputs.clone();
    let mut mg = ModuleGraph::new(resolver, config);
    println!("inputs {:?}", inputs);
    for path in inputs {
        let resource_path = path.as_path().absolutize();
        mg.resolve_entry_module(Some(resource_path.to_str().unwrap_or_default().to_string()));
        let resource_path_str = resource_path.to_str().unwrap();
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
            println!("output {}", output_path);
            let result = compile(&resolved_path, &mut mg);
            assets.output(&output_path, result)
        }
    }
}
