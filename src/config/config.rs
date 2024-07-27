use std::fs::write;

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
}

// TODO: parse tsconfig here
impl Config {
    pub fn new(options: ConfigOptions) -> Config {
        Self { options }
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
