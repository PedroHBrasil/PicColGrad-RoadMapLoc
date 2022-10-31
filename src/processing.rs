use image::DynamicImage;

/// Processes an image.
pub fn process_img(mut img: DynamicImage) -> DynamicImage {
    img.invert();

    img
}
