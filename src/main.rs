use crate::barcode_detector::PixelValue;
use std::env;
use std::process;
use std::time::Instant;

mod barcode_detector;
mod barcode_translate;
mod color_line_helpers;

type BarcodeBarArray = ([usize; 5], [[u8; 4]; 6], [[u8; 4]; 6]);

impl PixelValue for image::GrayImage {
    fn get_row_values(&self, y: u32, row: &mut [u8]) {
        let width = self.width() as usize;
        row.copy_from_slice(&self.as_raw()[y as usize * width ..][..width]);
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <image-file>", args[0]);
        process::exit(1);
    }
    load_image(&args[1]);
}

fn load_image(filename: &str) {
    let start = Instant::now();
    let img = match image::open(&filename) {
        Ok(img) => img,
        Err(e) => {
            eprintln!("Could not open image {}: {}", filename, e);
            process::exit(1);
        }
    };
    println!("Image opened in {:?}", start.elapsed());

    let luma = img.to_luma8();
    let dim = (luma.width(), luma.height());
    let barcodes = barcode_detector::process_image_by_rows(&luma, dim);
    for barcode in barcodes {
        match barcode_translate::translate_bar_code(&barcode) {
            Some(code) => {
                println!("{:?}", code);
            },
            None => {}
        }
    }
    println!("Image processed in {:?}", start.elapsed());
}
