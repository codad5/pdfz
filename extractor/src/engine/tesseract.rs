use std::{collections::HashMap, io::Read};
use std::path::Path;
use lopdf::{xobject::PdfImage, Document};
use image::{ImageBuffer, Rgba, RgbaImage, ImageFormat, ImageReader};
use std::io::Cursor;
use flate2::read::ZlibDecoder;
use rusty_tesseract::{self, Args, Image};
use std::future::Future;
use crate::types::engine_handler::PageExtractInfo;
use crate::{helper::file_helper, libs::redis::mark_progress, types::engine_handler::EngineHandler, worker::NewFileProcessQueue};

#[derive(Clone)]
pub struct TesseractEngine;

impl EngineHandler for TesseractEngine {
    fn new() -> Self {
        TesseractEngine
    }

    fn handle<'a>(&'a self, process_queue: &'a NewFileProcessQueue) -> impl Future<Output = Result<Vec<PageExtractInfo>, Box<dyn std::error::Error + Send + Sync>>> + Send + 'a {
        async move {
            Self::extract_file(process_queue.clone()).await
        }
    }
}



impl TesseractEngine {
     async fn extract_file(process_queue: NewFileProcessQueue) -> Result<Vec<PageExtractInfo>, Box<dyn std::error::Error + Send + Sync>> {
        println!("Extracting file {}", process_queue.file);
        let path = file_helper::get_upload_path(format!("{}", process_queue.file).as_str());
        println!("Processing {:?}", path);
        
        if !path.exists() {
            return Err(format!("File does not exist: {:?}", path).into());
        }

        let doc = Document::load(path).map_err(|e| format!("Error loading PDF file: {}", e))?;
        let mut page_count = 0;
        let file_id = process_queue.file.split('.').next().unwrap_or("");
        let mut all_page_info: Vec<PageExtractInfo> = Vec::new();

        for (page_num, page_id) in doc.get_pages() {
            if page_count > process_queue.page_count {
                break;
            }
            if process_queue.start_page > page_num {
                continue;
            }
            
            println!("Extracting page {}", page_num);
            let page_info: PageExtractInfo = Self::process_page(&doc, page_num, page_id);
            println!("Extracted page {} with {} images", page_num, page_info.image_path.len());
            
            mark_progress(file_id, page_num, process_queue.page_count).await?;
            all_page_info.push(page_info);
            page_count += 1;
        }
        
        Ok(all_page_info)
    }

    fn process_page(doc: &Document, page_num: u32, page_id: (u32, u16)) -> PageExtractInfo {
        let mut image_paths: Vec<String> = vec![];
        let mut image_text: Vec<String> = vec![];
        let mut text_map: Vec<String> = Vec::new();

        if let Ok(text_content) = doc.extract_text(&[page_num]) {
            text_map.push(text_content);
        }

        if let Ok(page_images) = doc.get_page_images(page_id) {
            for (i, image) in page_images.iter().enumerate() {
                let image_name = format!("{}_{}.png", page_num, i);
                let image_path = file_helper::get_pdf_image_process_path(image_name.as_str());
                
                if Self::save_pdf_image(&image, image_path.to_str().unwrap()).is_ok() {
                    if let Ok(extracted_text) = Self::extract_text_from_image(image_path.to_str().unwrap()) {
                        image_text.push(extracted_text);
                        image_paths.push(image_name);
                    }
                }
            }
        }

        PageExtractInfo {
            page_num,
            text: text_map.join(" "),
            image_path: image_paths,
            image_text,
        }
    }
    
    fn save_pdf_image<'a>(pdf_image: &PdfImage<'a>, file_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(filters) = &pdf_image.filters {
            for filter in filters {
                match filter.as_str() {
                    "DCTDecode" => {
                        let img = ImageReader::with_format(Cursor::new(pdf_image.content), ImageFormat::Jpeg)
                            .decode()
                            .map_err(|e| format!("Failed to decode JPEG image: {}", e))?;
                        img.save(file_name)?;
                        return Ok(());
                    }
                    "FlateDecode" => {
                        let mut decoder = ZlibDecoder::new(Cursor::new(pdf_image.content));
                        let mut decompressed_data = Vec::new();
                        decoder.read_to_end(&mut decompressed_data)?;

                        let img = ImageReader::new(Cursor::new(decompressed_data))
                            .with_guessed_format()?
                            .decode()
                            .map_err(|e| format!("Failed to decode FlateDecode image: {}", e))?;

                        let img = img.rotate90();
                        img.save(file_name)?;
                        return Ok(());
                    }
                    _ => return Err(format!("Unsupported filter: {}", filter).into()),
                }
            }
        }

        match pdf_image.color_space.as_deref() {
            Some("DeviceRGB") => {
                let expected_size = (pdf_image.width * pdf_image.height * 3) as usize;
                if pdf_image.content.len() != expected_size {
                    return Err("Content length does not match expected size for RGB".into());
                }

                let img: RgbaImage = ImageBuffer::from_fn(
                    pdf_image.width as u32,
                    pdf_image.height as u32,
                    |x, y| {
                        let idx = (y * pdf_image.width as u32 + x) as usize * 3;
                        Rgba([
                            pdf_image.content[idx],
                            pdf_image.content[idx + 1],
                            pdf_image.content[idx + 2],
                            255,
                        ])
                    },
                );
                img.save(file_name)?;
            }
            Some("DeviceGray") => {
                let expected_size = (pdf_image.width * pdf_image.height) as usize;
                if pdf_image.content.len() != expected_size {
                    return Err("Content length does not match expected size for Gray".into());
                }

                let img: RgbaImage = ImageBuffer::from_fn(
                    pdf_image.width as u32,
                    pdf_image.height as u32,
                    |x, y| {
                        let idx = (y * pdf_image.width as u32 + x) as usize;
                        let gray = pdf_image.content[idx];
                        Rgba([gray, gray, gray, 255])
                    },
                );
                img.save(file_name)?;
            }
            _ => return Err("Unsupported color space or image type".into()),
        }

        Ok(())
    }

    fn extract_text_from_image(image_path: &str) -> Result<String, Box<dyn std::error::Error>> {
        let mut img = ImageReader::open(image_path)?.decode()?;
        img = img.grayscale();
        let img = Image::from_dynamic_image(&img)?;
        let arg = Args::default();
        Ok(rusty_tesseract::image_to_string(&img, &arg)?)
    }
}


