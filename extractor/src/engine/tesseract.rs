use image::ImageReader;
use rusty_tesseract::{Image, Args, TessResult};
use std::{error::Error, future::Future, pin::Pin};

use crate::types::engine_handler::EngineHandler;

#[derive(Debug, Clone)]
pub struct TesseractEngine;

impl EngineHandler for TesseractEngine {
    fn new(_model: Option<String>) -> Self {
        TesseractEngine
    }

    fn extract_text_from_image(&self, image_path: String) 
        -> Pin<Box<dyn Future<Output = Result<String, Box<dyn Error + Send>>> + Send>> 
    {
        let image_path = image_path.to_owned();
        Box::pin(async move {
            tokio::task::spawn_blocking(move || -> Result<String, Box<dyn Error + Send>> {
                let img = ImageReader::open(&image_path)
                    .map_err(|e| Box::new(e) as Box<dyn Error + Send>)?  // Convert image error
                    .decode()
                    .map_err(|e| Box::new(e) as Box<dyn Error + Send>)?  // Convert decode error
                    .grayscale();
                
                let tesseract_img = Image::from_dynamic_image(&img)
                    .map_err(|e| Box::new(e) as Box<dyn Error + Send>)?;  // Convert Tesseract image error
                
                let args = Args::default();
                rusty_tesseract::image_to_string(&tesseract_img, &args)
                    .map_err(|e| Box::new(e) as Box<dyn Error + Send>)  // Convert Tesseract OCR error
            })
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error + Send>)?  // Convert JoinError
        })
    }
}