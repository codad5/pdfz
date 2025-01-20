use std::path::{Path, PathBuf};
use std::fs;

pub fn get_upload_path(file: &str) -> PathBuf {
    let base_path = std::env::var("SHARED_STORAGE_PATH").unwrap();
    let path = Path::new(&base_path).join(format!("upload/{}", file));

    // Ensure the parent directory exists, create if it doesn't
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).expect("Failed to create parent directory for upload path");
        }
    }

    path
}

pub fn get_pdf_image_process_path(file: &str) -> PathBuf {
    let base_path = std::env::var("SHARED_STORAGE_PATH").unwrap();
    let path = Path::new(&base_path).join(format!("pdf_image_process/{}", file));

    // Ensure the parent directory exists, create if it doesn't
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).expect("Failed to create parent directory for pdf image process path");
        }
    }

    path
}
