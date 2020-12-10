use std::path::Path;

/// Retrieves the parent path without file name given [Path].
/// If any error occurs, this function panics with additional message.
pub fn dir_name(path: &Path) -> String {
  path
    .parent()
    .expect("retrieving parent path failed")
    .to_str()
    .expect("converting parent path to string failed")
    .to_string()
}

/// Retrieves the file name with extension from given [Path].
/// If any error occurs, this function panics with additional message.
pub fn file_name(path: &Path) -> String {
  path
    .file_name()
    .expect("retrieving file name failed")
    .to_str()
    .expect("converting file name to string failed")
    .to_string()
}
