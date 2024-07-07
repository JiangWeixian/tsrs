use std::sync::Mutex;

use once_cell::sync::Lazy;

use super::ImportSpecifier;

pub static SHARED_IMPORTS: Lazy<Mutex<Vec<ImportSpecifier>>> = Lazy::new(|| Mutex::default());
