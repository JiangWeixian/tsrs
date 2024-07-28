use std::{
    fs::{create_dir_all, write},
    path::Path,
};

use sugar_path::SugarPath;
use swc_core::base::TransformOutput;

#[derive(Default, Debug)]
pub struct Assets {}

impl Assets {
    pub fn new() -> Assets {
        Self {}
    }
    // TODO: ensure output dir
    pub fn output(&self, output_path: &str, output: TransformOutput) {
        let path = output_path
            .as_path()
            .with_extension("")
            .with_extension("js");
        if let Some(parent) = path.parent() {
            create_dir_all(parent).expect("Failed to create directories");
        }
        let content = output.code;
        write(path, content).expect("Failed to write file");
    }
}
