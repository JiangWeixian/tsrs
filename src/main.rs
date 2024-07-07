mod compiler;
mod plugins;
mod resolver;
mod utils;
use std::{env, fs, path::Path};

use compiler::{transform, SwcCompiler};
use resolver::resolve;
use swc_core::base::config::Options;
use utils::ImportSpecifier;

fn main() {
    let options = Options::default();
    let resource_path = Path::new("./fixtures/index.js");
    let source = fs::read_to_string(resource_path).expect("failed to read file");
    let c = SwcCompiler::new(resource_path.to_path_buf(), source.clone(), options)
        .map_err(|err| anyhow::Error::from(err))
        .expect("TODO:");
    let options = c.options();
    let top_level_mark = options
        .top_level_mark
        .expect("`top_level_mark` should be initialized");
    let unresolved_mark = options
        .unresolved_mark
        .expect("`unresolved_mark` should be initialized");

    let mut collected_imports: Vec<ImportSpecifier> = vec![];
    let built = c
        .parse(None, |_| {
            transform(
                &resource_path,
                Some(c.comments()),
                top_level_mark,
                unresolved_mark,
                c.cm().clone(),
                &mut collected_imports,
            )
        })
        .expect("TODO:");
    let program = c
        .transform(built)
        .map_err(|err| anyhow::Error::from(err))
        .expect("TODO:");
    for decl in collected_imports.iter() {
        let resolved_path = resolve(
            String::from(env::current_dir().unwrap().to_str().unwrap()),
            decl.clone().n.unwrap(),
        );
        println!("Hello, world! {:?}", resolved_path);
    }
}
