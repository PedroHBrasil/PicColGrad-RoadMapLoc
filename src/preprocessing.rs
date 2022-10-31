use image::io::Reader as ImageReader;
use image::DynamicImage;
use serde::Deserialize;
use serde_json;
use std::fs;
use std::path::Path;

#[derive(Deserialize)]
pub struct Input {
    pub img_path: String,
}

pub fn load_input() -> Input {
    let json_path = Path::new("./data/input.json");
    let json_file = fs::read_to_string(json_path).unwrap();
    let input: Input = serde_json::from_str(&json_file).unwrap();

    input
}

pub fn load_image(img_path: &str) -> DynamicImage {
    ImageReader::open(img_path).unwrap().decode().unwrap()
}
