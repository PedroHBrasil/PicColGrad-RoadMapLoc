use image::DynamicImage;

/// Saves an image.
pub fn save_img(img: DynamicImage) -> () {
    img.save("./out/out_img.png").unwrap();
}
