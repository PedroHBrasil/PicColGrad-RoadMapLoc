use image::{ImageBuffer, LumaA};

/// Saves an image.
pub fn save_img(img: ImageBuffer<LumaA<u8>, Vec<u8>>, out_file_name: &String) -> () {
    let mut out_path = String::from("./out/");
    out_path.push_str(out_file_name);
    out_path.push_str(".png");
    img.save(out_path).unwrap();
}
