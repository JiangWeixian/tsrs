use std::fs::write;

use swc_core::base::TransformOutput;

#[derive(Default, Debug)]
pub struct Assets {}

impl Assets {
    pub fn new() -> Assets {
        Self {}
    }
    pub fn output(&self, output_path: &str, output: TransformOutput) {
        let content = output.code;
        write(output_path, content).expect("TODO: handle write failed");
    }
}
