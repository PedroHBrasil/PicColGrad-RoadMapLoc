use image::io::Reader as ImageReader;
use image::DynamicImage;
use serde::Deserialize;
use serde_json;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
/// Contains all the program's inputs.
/// Use as in function load_input.
pub struct Input {
    /// Path to image file.
    pub img_path: String,
    /// Number of gray shades to interpret in image
    pub n_shades: u8,
    /// Number of gradient directions to evaluate
    pub n_grad_dir: u32,
}

/// Loads the inputs from the json file, storing them on an Input struct.
pub fn load_input() -> Input {
    let json_path = Path::new("./data/input.json");
    let json_file = fs::read_to_string(json_path).unwrap();
    let input: Input = serde_json::from_str(&json_file).unwrap();

    input
}

/// Loads an image given its address.
pub fn load_image(img_path: &str) -> DynamicImage {
    ImageReader::open(img_path).unwrap().decode().unwrap()
}
