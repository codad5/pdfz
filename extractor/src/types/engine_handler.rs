use std::{collections::HashMap, sync::Arc};
use lapin::message;
use redis::Client;
use tokio::{sync::Semaphore, task};
use std::future::Future;
use crate::{engine::tesseract::TesseractEngine, helper::file_helper::save_processed_json, libs::redis::{mark_as, Status}, worker::NewFileProcessQueue};



#[derive(Debug, Clone, serde::Serialize)]
pub struct PageExtractInfo {
    pub page_num: u32,
    pub text:  String,
    pub image_path: Vec<String>,
    pub image_text: Vec<String>,
}
pub enum Engines {
    Tesseract
}

impl Engines {
    pub fn from(engine: &str) -> Option<Self> {
        match engine.to_lowercase().as_str() {
            "tesseract" => Some(Engines::Tesseract),
            _ => None,
        }
    }

    pub fn get_handler(&self) -> impl EngineHandler {
        match self {
            Engines::Tesseract => TesseractEngine::new(),
        }
    }

    pub async fn handle(&self, message: NewFileProcessQueue, redis_client: &Arc<Client>, semaphore: &Arc<Semaphore>) {
        let redis_client = redis_client.clone();
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let engine = self.get_handler();
        task::spawn(async move {
            println!("Processing file: {}", message.file);
            let id = message.file.split('.').next().unwrap_or("");
            let result = engine.handle(&message).await;
            let result = match result {
                Ok(res) => {
                    save_processed_json(res, id);
                    if let Err(e) = mark_as(&redis_client, id, Status::Done).await {
                        eprintln!("Error marking as success: {}", e);
                    }
                },
                Err(e) => {
                    eprintln!("Error processing file: {}", e);
                    if let Err(e) = mark_as(&redis_client, id, Status::Failed).await {
                        eprintln!("Error marking as failed: {}", e);
                    }
                    return;
                }
            };
            drop(permit);
        });
    }
}

pub trait EngineHandler: Send {
    fn new() -> Self where Self: Sized;
    fn handle<'a>(&'a self, message: &'a NewFileProcessQueue) -> impl Future<Output = Result<Vec<PageExtractInfo>, Box<dyn std::error::Error + Send + Sync>>> + Send + 'a;
}

