#![deny(clippy::all)]
use tswc::apis::{transform as tswc, TransformOptions};

#[macro_use]
extern crate napi_derive;

#[napi]
pub fn transform(options: TransformOptions) {
  return tswc(options);
}
