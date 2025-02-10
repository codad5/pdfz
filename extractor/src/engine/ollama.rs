use std::fs;
use std::pin::Pin;
use std::{collections::HashMap, io::Read};
use std::path::Path;
use base64::Engine;
use lopdf::{xobject::PdfImage, Document};
use image::{ImageBuffer, Rgba, RgbaImage, ImageFormat, ImageReader};
use ollama_rs::generation::completion::request::GenerationRequest;
use ollama_rs::generation::completion::GenerationResponse;
use ollama_rs::generation::images::Image;
use ollama_rs::Ollama;
use std::io::Cursor;
use flate2::read::ZlibDecoder;
use std::future::Future;
use crate::types::engine_handler::PageExtractInfo;
use crate::{helper::file_helper, libs::redis::mark_progress, types::engine_handler::EngineHandler, worker::NewFileProcessQueue};

const PROMPT: &str = "Only return the text you can see in this image";

#[derive(Debug, Clone)]
pub struct OllamaEngine {
    model: String
}

impl EngineHandler for OllamaEngine {
    fn new(model: Option<String>) -> Self {
        OllamaEngine {
            model: model.unwrap()
        }
    }

    fn extract_text_from_image(&self, image_path: String) 
        -> Pin<Box<dyn Future<Output = Result<String, Box<dyn std::error::Error + Send>>> + Send>> 
    {
        let model = self.model.clone();
        Box::pin(async move {
            let bytes = fs::read(image_path.as_str())
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?;

            let base64_image = base64::engine::general_purpose::STANDARD.encode(&bytes);
            
            let request = GenerationRequest::new(model, PROMPT.to_string())
                .add_image(Image::from_base64(&base64_image));

            let ollama = Ollama::default();
            let response = ollama.generate(request).await
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?;

            Ok(response.response)
        })
    }
}

async fn send_request<'a>(
    request: GenerationRequest<'a>,
) -> Result<GenerationResponse, Box<dyn std::error::Error>> {
    let ollama = Ollama::default();
    let res = ollama.list_local_models().await?;

    println!("all Models: {:?}", res);
    let response = ollama.generate(request).await?;
    Ok(response)
}