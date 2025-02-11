use std::{clone, collections::HashMap, error::Error, fmt::Debug, io::Cursor, sync::Arc};
use image::{ImageBuffer, ImageFormat, ImageReader, RgbaImage};
use lapin::message;
use lopdf::{xobject::PdfImage, Document};
use ollama_rs::Ollama;
use redis::Client;
use tokio::{sync::Semaphore, task};
use std::future::Future;
use crate::{engine::{ollama::OllamaEngine, tesseract::TesseractEngine, MainEngine}, helper::file_helper::{self, save_processed_json}, libs::redis::{ mark_as_done, mark_as_failed, mark_progress, Status}, worker::NewFileProcessQueue};
use std::pin::Pin;


#[derive(Debug, Clone, serde::Serialize)]
pub struct PageExtractInfo {
    pub page_num: u32,
    pub text:  String,
    pub image_path: Vec<String>,
    pub image_text: Vec<String>,
}
pub enum Engines {
    Tesseract,
    Ollama
}

impl Engines {
    pub fn from(engine: &str) -> Option<Self> {
        match engine.to_lowercase().as_str() {
            "tesseract" => Some(Engines::Tesseract),
            "ollama" => Some(Engines::Ollama),
            _ => None,
        }
    }

    pub fn get_handler(&self, model: Option<String>) -> Box<dyn EngineHandler> {
        match self {
            Engines::Ollama => Box::new(OllamaEngine::new(model)),
            Engines::Tesseract => Box::new(TesseractEngine::new(None)),
        }
    }


    pub async fn handle(&self, message: NewFileProcessQueue,semaphore: &Arc<Semaphore>) {
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let engine = self.get_handler(message.model.clone());
        task::spawn(async move {
            let main_handler = MainEngine::new(engine, message);
            main_handler.run().await;
            drop(permit);
        });
    }
}


pub trait EngineHandler: Send + Sync + Debug {
    fn new(model: Option<String>) -> Self where Self: Sized;
    
    fn extract_text_from_image(&self, image_path: String) 
        -> Pin<Box<dyn Future<Output = Result<String, Box<dyn Error + Send>>> + Send>>;
}
