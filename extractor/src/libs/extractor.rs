use std::collections::HashMap;
use std::path::Path;
use lopdf::{xobject::PdfImage, Document};
use image::{ImageBuffer, Rgba, RgbaImage, ImageFormat, ImageReader};
use std::io::Cursor;
use flate2::read::ZlibDecoder;
use std::io::prelude::*;
use rusty_tesseract::{self, Args, Image};

use crate::{helper::file_helper, worker::NewFileProcessQueue};



pub async fn extract_file(process_queue: NewFileProcessQueue) -> Result<(), Box<dyn std::error::Error>> {
    println!("Extracting file {}", process_queue.file);
    let path = file_helper::get_upload_path(format!("{}.pdf", process_queue.file).as_str());
    println!("Processing {:?}",  path);
    let doc = Document::load(path);
    if doc.is_err() {
        return Err(format!("Error loading PDF file: {}", doc.err().unwrap()).into());
    }
    let doc = doc.unwrap();
    // store image path in hash map with page number as key in testing for commit 
    let mut image_path: HashMap<u32, Vec<String>> = HashMap::new();
    // store text path in hash map with page number as key
    let mut text_path: HashMap<u32, String> = HashMap::new();
    let mut text = String::new();
    let mut page_count = 0;
    for (page_num, page_id) in doc.get_pages() {
        if page_count > process_queue.page_count {
            break;
        }
        if process_queue.start_page > page_num {
            continue;
        }
        let page_info = process_page(&doc, page_num, page_id);
        page_count += 1;
    }
    Ok(())
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct PageExtractInfo {
    page_num: u32,
    text: String,
    image_path: Vec<String>,
    image_text: Vec<String>,
}

pub fn process_page(doc : &Document, page_num: u32, page_id: (u32, u16)) -> PageExtractInfo {
    let mut image_paths:  Vec<String> = vec![];
    let mut image_text:  Vec<String> = vec![];
    let mut text = String::new();
    match doc.extract_text(&[page_num]) {
        Ok(text_content) => {
            // push to text with a line break the the page number first then the text
            // eg Page 1 - text
            text.push_str(&format!("{}\n", text_content));
        }
        Err(err) => {
            println!("Error extracting text from page {}: {}", page_num, err);
        }
    };

    let page_images = doc.get_page_images(page_id);
    if page_images.is_ok() {
        let page_images = page_images.unwrap();
        for (i, image) in page_images.iter().enumerate() {
            let image_name = format!("{}_{}.png", page_num, i);
            let image_path = file_helper::get_pdf_image_process_path(image_name.as_str());
            if save_pdf_image(&image, image_path.to_str().unwrap()).is_ok() {
                image_text.push(extract_text_from_image(image_path.to_str().unwrap()).unwrap());
                image_paths.push(image_name.as_str().to_string());
            }
        }
    }

    return PageExtractInfo {
        page_num,
        text,
        image_path: image_paths,
        image_text
    };

}


// Handle decoding of raw image data based on color space and content
pub fn save_pdf_image<'a>(pdf_image: &PdfImage<'a>, file_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Check if the image has any filters applied (such as DCTDecode for JPEGs)
    if let Some(filters) = &pdf_image.filters {
        for filter in filters {
            match filter.as_str() {
                "DCTDecode" => {
                    // This is a JPEG image, decode the JPEG data
                    println!("DCTDecode filter detected, decoding JPEG data...");
                    
                    let img = ImageReader::with_format(Cursor::new(pdf_image.content), ImageFormat::Jpeg)
                        .decode()
                        .map_err(|e| format!("Failed to decode JPEG image: {}", e))?;
                    
                    // Save the decoded image as PNG
                    img.save(file_name)?;
                    return Ok(());
                }
                "FlateDecode" => {
                    // Handle FlateDecode (likely PNG or ZIP-compressed data)
                    println!("FlateDecode filter detected (decompressing data)...");

                    // Decompress using Flate (zlib)
                    let mut decoder = ZlibDecoder::new(Cursor::new(pdf_image.content));
                    let mut decompressed_data = Vec::new();
                    decoder.read_to_end(&mut decompressed_data)?;

                    // Try to decode the decompressed data as an image
                    let img = ImageReader::new(Cursor::new(decompressed_data))
                        .with_guessed_format()?
                        .decode()
                        .map_err(|e| format!("Failed to decode FlateDecode image: {}", e))?;

                    let img = img.rotate90();

                    // Save the decoded image as PNG
                    img.save(file_name)?;
                    return Ok(());
                }
                _ => {
                    return Err(format!("Unsupported filter: {}", filter).into());
                }
            }
        }
    }

    // If there are no filters, handle color spaces like RGB and Grayscale
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
    let mut img = ImageReader::open(image_path).unwrap().decode().unwrap();
    img = img.grayscale();
    let img = Image::from_dynamic_image(&img).unwrap();
    let arg = Args::default();
    Ok(rusty_tesseract::image_to_string(&img, &arg).unwrap())
}
