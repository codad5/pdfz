use std::fs;
use std::pin::Pin;
use base64::Engine;
use ollama_rs::generation::completion::request::GenerationRequest;
use ollama_rs::generation::images::Image;
use ollama_rs::Ollama;
use std::future::Future;
use crate::types::engine_handler::EngineHandler;

const PROMPT: &str = "Please perform OCR on the supplied image and output the extracted text exactly as it appears. If the image contains multiple columns or sections, preserve the structural layout as much as possible. Do not include any explanations, commentary, or formatting modifications.";

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
            println!("reached here from ollama methid");
            let bytes = fs::read(image_path.as_str())
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?;

            let base64_image = base64::engine::general_purpose::STANDARD.encode(&bytes);
            
            let request = GenerationRequest::new(model, PROMPT.to_string())
                .add_image(Image::from_base64(&base64_image));

            let base_host = std::env::var("OLLAMA_BASE_HOST")
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?;
            let base_port = std::env::var("OLLAMA_BASE_PORT")
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?
                .parse::<u16>()
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?;

            let ollama = Ollama::new(base_host, base_port);
            let response = ollama.generate(request).await
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send>)?;
            Ok(response.response)
        })
    }
}

