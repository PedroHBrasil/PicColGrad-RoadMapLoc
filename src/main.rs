use image::io::Reader as ImageReader;
use serde_json::{self, Value};
use std::fs;
use std::path::Path;

fn main() {
    // Loads json file
    let json_path = Path::new("./data/input.json");
    let json_file = fs::read_to_string(json_path).unwrap();
    let json_content: Value = serde_json::from_str(&json_file).unwrap();
    // Loads image from json file's path
    let img_path = json_content["img_path"].as_str().unwrap();
    let mut img = ImageReader::open(img_path).unwrap().decode().unwrap();
    // TEST Processes image
    img.invert();
    // TEST Saves resulting image
    img.save("./out/out_img.png").unwrap();
}
