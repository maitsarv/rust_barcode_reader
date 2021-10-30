use crate::barcode_detector::PixelValue;
use image::{DynamicImage, GenericImageView};
use std::env;
use std::time::Instant;

mod barcode_detector;
mod barcode_translate;
mod color_line_helpers;

type BarcodeBarArray = ([usize; 5], [[u8; 4]; 6], [[u8; 4]; 6]);

impl PixelValue for DynamicImage {
    fn get_pixel_value(&self, x: u32, y: u32, channel: usize, w: usize) -> u8 {
        return self.get_pixel(x, y).0[channel];
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    load_image(&args[1]);
}

fn load_image(filename: &String) {
    println!("Start: {:?}", Instant::now());
    let img = image::open(&filename).unwrap();
    println!("Image opened: {:?}", Instant::now());

    let dim = img.dimensions();
    let barcodes = barcode_detector::process_image_by_rows(&img, dim, 0);
    for barcode in barcodes {
        match barcode_translate::translate_bar_code(&barcode) {
            Some(code) =>  {
                println!("{:?}", code);
            },
            None => {}
        }
    }
    println!("Image processed: {:?}", Instant::now());
}
