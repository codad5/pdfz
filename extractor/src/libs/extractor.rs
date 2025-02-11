use std::collections::HashMap;
use std::path::Path;
use lopdf::{xobject::PdfImage, Document};
use image::{ImageBuffer, Rgba, RgbaImage, ImageFormat, ImageReader};
use std::io::Cursor;
use flate2::read::ZlibDecoder;
use std::io::prelude::*;
use rusty_tesseract::{self, Args, Image};

use crate::{helper::file_helper, libs::redis::mark_progress, types::engine_handler::PageExtractInfo, worker::NewFileProcessQueue};

