use std::path::PathBuf;

pub fn find_up_dir(context: PathBuf) -> Option<String> {
  if context.is_dir() {
    context.to_str().map(|f| f.to_string())
  } else {
    context
      .parent()
      .and_then(|parent| find_up_dir(parent.to_path_buf()))
  }
}
