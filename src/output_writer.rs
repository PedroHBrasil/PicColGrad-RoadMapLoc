use image::{ImageBuffer, LumaA};

/// Saves an image.
pub fn save_img(img: ImageBuffer<LumaA<u8>, Vec<u8>>) -> () {
    img.save("./out/out_img.png").unwrap();
}
