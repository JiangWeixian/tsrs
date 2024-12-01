use log::debug;
use std::path::{Path, PathBuf};
use sugar_path::SugarPath;

use ignore::{overrides::OverrideBuilder, WalkBuilder};
use tsconfig::TsConfig;

#[derive(Debug)]
pub struct ConfigOptions {
  pub output: Option<String>,
  pub root: PathBuf,
  pub exclude: Option<Vec<String>>,
  pub packages: Vec<String>,
}

impl Default for ConfigOptions {
  fn default() -> Self {
    Self {
      output: None,
      root: PathBuf::default(),
      exclude: None,
      packages: Default::default(),
    }
  }
}

#[derive(Default, Debug)]
pub struct ResolvedConfigOptions {
  pub input: PathBuf,
  pub output: PathBuf,
  pub root: PathBuf,
  pub exclude: Vec<String>,
  pub packages: Vec<String>,
}

#[derive(Default, Debug)]
pub struct Config {
  pub options: ConfigOptions,
  pub resolved_options: ResolvedConfigOptions,
  pub files: Vec<PathBuf>,
  pub tsconfig: Option<TsConfig>,
  pub packages: Vec<String>,
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
    let mut input: PathBuf = self.options.root.clone();
    let mut exclude = self.options.exclude.clone().unwrap_or(vec![]);
    let default_output: PathBuf = input.join("dist");
    let mut output = default_output.clone();

    if let Some(tsconfig) = &self.tsconfig {
      let include = { tsconfig.include.as_ref().and_then(|f| f.get(0)) };
      input = if let Some(include) = include {
        self.options.root.join(include)
      } else {
        self.options.root.clone()
      };
      output = if let Some(out) = &self.options.output {
        out.as_path().absolutize()
      } else {
        tsconfig
          .compiler_options
          .as_ref()
          .and_then(|f| f.out_dir.as_ref())
          .and_then(|f| Some(self.options.root.join(f)))
          .unwrap_or(default_output)
      };

      exclude = if let Some(exclude) = &self.options.exclude {
        exclude.clone()
      } else {
        tsconfig
          .exclude
          .as_ref()
          .unwrap_or(&Vec::<String>::new())
          .to_vec()
      };
    }
    let resolved_options = ResolvedConfigOptions {
      input,
      output,
      exclude,
      root: self.options.root.clone(),
      packages: self.options.packages.clone(),
    };
    self.resolved_options = resolved_options;
  }
  pub fn search_files(&mut self) {
    let root = &self.resolved_options.input;
    let mut ignores = self
      .resolved_options
      .exclude
      .clone()
      .into_iter()
      .map(|f| format!("!{}", f))
      .collect::<Vec<String>>();
    let mut override_builder = OverrideBuilder::new(root.as_path());
    // TODO: ext should configable
    let mut globs: Vec<String> = vec![
      "**/*.ts".into(),
      "**/*.tsx".into(),
      "**/*.js".into(),
      "**/*.jsx".into(),
      "!node_modules".into(),
      "!**/*.d.ts".into(),
    ];
    globs.append(&mut ignores);
    debug!(target: "tswc", "globs {:?}", globs);
    for gb in globs {
      override_builder.add(&gb).unwrap();
    }
    let override_builder = override_builder.build();
    if let Ok(ob) = override_builder {
      let mut builder = WalkBuilder::new(root.as_path());
      debug!(target: "tswc", "search files on {:?}", root);
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
            self.files.push(entry.path().to_path_buf())
          }
        }
      }
    };
  }
  pub fn parse_tsconfig(&mut self, tsconfig_file_path: &Path) {
    if tsconfig_file_path.exists() {
      let config = TsConfig::parse_file(&tsconfig_file_path).ok();
      debug!(target: "tswc", "tsconfig {:?}", config);
      self.tsconfig = config;
    } else {
      let config = TsConfig::parse_str(
        r#"
{
  "compilerOptions": {
    "target": "ES2021",
    "module": "CommonJS",
    "moduleResolution": "Node"
  }
}
      "#,
      )
      .ok();
      debug!(target: "tswc", "tsconfig {:?}", config);
      self.tsconfig = config;
    }
  }
}
