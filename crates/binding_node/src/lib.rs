#![deny(clippy::all)]
use tswc::apis::{transform as tswc, TransformOptions, pre_optimize as tswc_pre_optimize, PreOptimizeOptions};

#[macro_use]
extern crate napi_derive;

#[napi]
pub fn transform(options: TransformOptions) {
  return tswc(options);
}

#[napi]
pub fn pre_optimize(options: PreOptimizeOptions) {
  return tswc_pre_optimize(options);
}
