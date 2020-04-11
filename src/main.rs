use std::env;
use image::{GenericImageView, DynamicImage};
use std::time::Instant;
use crate::barcode_detector::PixelValue;

mod barcode_detector;

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
    barcode_detector::process_by_rows_image(&img,dim,0);
    println!("Image processed: {:?}",Instant::now());
}