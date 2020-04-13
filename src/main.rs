use std::env;
use image::{GenericImageView, DynamicImage};
use std::time::Instant;
use crate::barcode_detector::PixelValue;

mod barcode_translate;
mod barcode_detector;

type BarcodeBarArray = ([usize; 4], [[u8; 4]; 6], [[u8; 4]; 6]);

impl PixelValue for DynamicImage{
    fn get_pixel_value(&self, x: u32, y:u32, channel: usize,  w:usize) -> u8{
        return self.get_pixel(x, y).0[channel]
    }
}


fn main() {
    let args: Vec<String> = env::args().collect();
    load_image(&args[1]);
}

fn load_image(filename: &String) {
    println!("Start: {:?}",Instant::now());
    let img = image::open(&filename).unwrap();
    println!("Image opened: {:?}",Instant::now());

    let dim = img.dimensions();
    let barcodes = barcode_detector::process_image_by_rows(&img,dim,0);
    for barcode in barcodes{
        barcode_translate::translate_bar_code(&barcode);
    }
    println!("Image processed: {:?}",Instant::now());
}
