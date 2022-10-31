use image::DynamicImage;

pub fn save_img(img: DynamicImage) -> () {
    img.save("./out/out_img.png").unwrap();
}
