use std::path::{Path, PathBuf};
use std::fs;

use crate::types::engine_handler::PageExtractInfo;


pub fn get_upload_path(file: &str) -> PathBuf {
    let base_path = std::env::var("SHARED_STORAGE_PATH").unwrap();
    let folder_path = Path::new(&base_path);

    if folder_path.exists() {
        println!("folder exist {:?}", folder_path);
    }

    let folder_path = folder_path.join("upload/pdf");
    println!("Folder path: {:?}", folder_path);

    // Ensure the folder exists before using it
    if !folder_path.exists() {
        println!("folder does not exisit {:?}", folder_path);
        fs::create_dir_all(&folder_path).expect("Failed to create upload directory");
    }

    // Now construct the file path
    let path = folder_path.join(file);
    // ;ist all the child dir in the folder
    let paths = fs::read_dir(&folder_path).unwrap();
    for path in paths {
        println!("Name: {:?}", path.unwrap().path());
    }
    if path.exists() {
        println!("file exist {:?}", path);
        println!("is file {:?}", path.is_file());
    }
    path
}

pub fn get_pdf_image_process_path(file: &str) -> PathBuf {
    let base_path = std::env::var("SHARED_STORAGE_PATH").unwrap();
    let folder_path = Path::new(&base_path).join("pdf_image_process");

    println!("Folder path: {:?}", folder_path);

    // Ensure the folder exists before using it
    if !folder_path.exists() {
        fs::create_dir_all(&folder_path).expect("Failed to create pdf image process directory");
    }

    // Now construct the file path
    let path = folder_path.join(file);
    path
}


pub fn save_processed_json(data : Vec<PageExtractInfo>, file_id : &str){
    let base_path = std::env::var("SHARED_STORAGE_PATH").unwrap();
    let folder_path = Path::new(&base_path).join("processed");
    if !folder_path.exists() {
        fs::create_dir_all(&folder_path)
            .expect("Failed to create processed directory");

    }
    // Extract the file ID without extension if needed
    let clean_id = file_id.split('.').next().unwrap_or(file_id);
      
    // Create the full file path for the JSON
    let json_path = folder_path.join(format!("{}.json", clean_id));
    
     let json_content = serde_json::to_string_pretty(&data)
        .expect( "Failed to serialize data");

    // Write the JSON content to the file
    fs::write(&json_path, json_content).expect("Failed to write JSON file");

    println!("Processed JSON saved to {:?}", json_path);

}