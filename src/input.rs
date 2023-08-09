//! Path processing functions.

use std::path::{PathBuf, Path};

pub const EXPECT_UTF8_PATH: &str = "expected path to be a valid utf-8 string";

pub fn file_name(path: &PathBuf) -> &str {
    path.file_name()
        .expect("Expected path to end in a file name")
        .to_str()
        .expect(EXPECT_UTF8_PATH)
}

pub fn child_path<P>(base: &PathBuf, suffix: P) -> PathBuf
where
    P: AsRef<Path>,
{
    let mut result = base.clone();
    result.push(suffix);
    result
}

pub fn sanitize_path(path_dir: &mut PathBuf, base_dir: &PathBuf) -> PathBuf {
    path_dir.pop();
    path_dir.strip_prefix(&base_dir)
        .expect("Expected `base_dir` to be a valid base for `path_dir`")
        .to_path_buf()
}

pub fn path_buf_to_key_name(path_buf: &PathBuf) -> String {
    let mut key_name = String::from("/");
    key_name.push_str(path_buf.to_str().expect(EXPECT_UTF8_PATH));
    key_name.replace("\\", "/")
}
