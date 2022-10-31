use image::{ImageBuffer, LumaA};

pub fn save_img(img: ImageBuffer<LumaA<u16>, Vec<u16>>) -> () {
    img.save("./out/out_img.png").unwrap();
}
