use std::{
    fs::write,
    path::{Path, PathBuf},
};

use ignore::{overrides::OverrideBuilder, WalkBuilder};
use sugar_path::SugarPath;
use swc_core::base::TransformOutput;

#[derive(Default, Debug)]
pub struct ConfigOptions {
    pub input: String,
    pub output: String,
}

#[derive(Default, Debug)]
pub struct Config {
    pub options: ConfigOptions,
    pub inputs: Vec<PathBuf>,
}

// TODO: parse tsconfig here
impl Config {
    pub fn new(options: ConfigOptions) -> Config {
        Self {
            options,
            ..Default::default()
        }
    }
    pub fn search_files(&mut self) {
        let mut override_builder = OverrideBuilder::new(self.options.input.as_path());
        // TODO: ext should configable
        let globs = vec![
            "**/*.ts",
            "**/*.tsx",
            "**/*.js",
            "**/*.jsx",
            "!node_modules",
        ];
        for gb in globs {
            override_builder.add(gb).unwrap();
        }
        let override_builder = override_builder.build();
        if let Ok(ob) = override_builder {
            let mut builder = WalkBuilder::new(self.options.input.as_path());
            builder.overrides(ob);
            builder.standard_filters(true);
            let walker = builder.build();
            for entry in walker {
                if let Ok(entry) = entry {
                    let is_file = entry
                        .file_type()
                        .and_then(|f| Some(f.is_file()))
                        .unwrap_or(false);
                    if is_file {
                        self.inputs.push(entry.path().to_path_buf())
                    }
                }
            }
        };
    }
    pub fn normalize_output_path(&self, file_path: &str) -> String {
        let normalized = file_path
            .replace(&self.options.input, &self.options.output)
            .as_path()
            .with_extension("")
            .with_extension("js");
        println!("output_path {:?}", normalized);
        normalized.to_str().map(|f| f.to_string()).unwrap()
    }
    pub fn output(&self, file_path: &str, output: TransformOutput) {
        let content = output.code;
        let output_path = self.normalize_output_path(file_path);
        write(output_path, content).expect("TODO: handle write failed");
    }
}
