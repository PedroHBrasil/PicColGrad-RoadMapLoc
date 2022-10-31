use image::{DynamicImage, ImageBuffer, LumaA};

pub fn process_img(img: DynamicImage) -> ImageBuffer<LumaA<u16>, Vec<u16>> {
    let img_gs = img.into_luma_alpha16();

    img_gs
}
