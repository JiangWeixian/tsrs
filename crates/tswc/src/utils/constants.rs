use once_cell::sync::Lazy;
use regex::Regex;

pub static QUERY_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\?.*").expect("query re init failed"));
pub static SCRIPT_RE: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"\.ts$|\.tsx$|\.js$|\.jsx$").expect("ext re init failed"));
