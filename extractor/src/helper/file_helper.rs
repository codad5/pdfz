use std::path::{Path, PathBuf};

pub fn get_upload_path(file : &str) -> PathBuf {
    let base_path = std::env::var("SHARED_STORAGE_PATH").unwrap();
    let path = Path::new(base_path.as_str()).join(format!("upload/{}", file));
    path
}