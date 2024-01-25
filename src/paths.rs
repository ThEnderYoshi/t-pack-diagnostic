//! Contains some path processing functions.

use std::path::{PathBuf, Path};

pub const EXPECT_UTF8_PATH: &str = "expected path to be a valid utf-8 string";

/// Returns the file name at the end of the path.
/// 
/// ## Panics
/// 
/// Panics if the path does not have an file name.
#[inline]
pub fn file_name(path: &PathBuf) -> &str {
    path
        .file_name()
        .expect("expected path to end in a file name")
        .to_str()
        .expect(EXPECT_UTF8_PATH)
}

///// Returns the file extension at the end of the path.
///// 
///// ## Panics
///// 
///// Panics if the path does not have an extension.
//#[inline]
//pub fn extension(path: &PathBuf) -> &str {
//    path
//        .extension()
//        .expect("expected path to have a file extension")
//        .to_str()
//        .expect(EXPECT_UTF8_PATH)
//}

/// Returns the child `suffix` of the path `base`.
#[inline]
pub fn push<P>(base: &PathBuf, suffix: P) -> PathBuf where P: AsRef<Path> {
    let mut result = base.clone();
    result.push(suffix);
    result
}

/// Prepares a path to be added to a reference file.
#[inline]
pub fn sanitize_path(mut path_dir: PathBuf, base_dir: &PathBuf) -> PathBuf {
    path_dir.pop();
    path_dir.strip_prefix(&base_dir)
        .expect("expected `base_dir` to be a valid base for `path_dir`")
        .to_path_buf()
}

pub fn path_buf_to_key_name(path_buf: &PathBuf) -> String {
    let mut key_name = String::from("/");
    key_name.push_str(path_buf.to_str().expect(EXPECT_UTF8_PATH));
    key_name.replace("\\", "/")
}
