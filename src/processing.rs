use image::DynamicImage;

pub fn process_img(mut img: DynamicImage) -> DynamicImage {
    img.invert();

    img
}
