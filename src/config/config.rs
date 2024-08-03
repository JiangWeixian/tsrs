use std::path::{Path, PathBuf};

use ignore::{overrides::OverrideBuilder, WalkBuilder};
use tsconfig::TsConfig;

#[derive(Default, Debug)]
pub struct ConfigOptions {
    pub input: String,
    pub output: Option<String>,
    pub root: PathBuf,
}

#[derive(Default, Debug)]
pub struct ResolvedConfigOptions {
    pub input: String,
    pub output: PathBuf,
    pub root: PathBuf,
}

#[derive(Default, Debug)]
pub struct Config {
    pub options: ConfigOptions,
    pub resolved_options: ResolvedConfigOptions,
    /// TODO: merge with tsconfig.include
    pub inputs: Vec<PathBuf>,
    pub tsconfig: Option<TsConfig>,
}

impl Config {
    pub fn new(options: ConfigOptions) -> Config {
        Self {
            options,
            ..Default::default()
        }
    }
    pub fn resolve_options(&mut self, tsconfig_file_path: &Path) {
        self.parse_tsconfig(tsconfig_file_path);
        let default_out_dir = String::from("./dist");
        let resolved_out_dir = self.options.output.as_ref().unwrap_or(&default_out_dir);
        let output = self
            .tsconfig
            .as_ref()
            .and_then(|f| f.compiler_options.as_ref())
            .and_then(|f| f.out_dir.as_ref())
            .unwrap_or(&resolved_out_dir);
        let output = self.options.root.join(output);
        let resolved_options = ResolvedConfigOptions {
            input: self.options.input.clone(),
            output,
            root: self.options.root.clone(),
        };
        self.resolved_options = resolved_options;
    }
    pub fn search_files(&mut self) {
        let includes = self
            .tsconfig
            .as_ref()
            .and_then(|f| f.include.clone())
            .unwrap_or(Default::default());
        // TODO: support array search;
        let include = includes.get(0);
        if let Some(include) = include {
            let root = self.resolved_options.root.join(include);
            let mut override_builder = OverrideBuilder::new(root.as_path());
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
                let mut builder = WalkBuilder::new(root.as_path());
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
        };
    }
    pub fn parse_tsconfig(&mut self, tsconfig_file_path: &Path) {
        let config = TsConfig::parse_file(&tsconfig_file_path).ok();
        self.tsconfig = config;
    }
}
