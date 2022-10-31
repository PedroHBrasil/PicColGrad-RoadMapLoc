use image::{DynamicImage, ImageBuffer, LumaA, Pixel};

/// Processes an image.
pub fn process_img(n_read_colors: u8, img: DynamicImage) -> ImageBuffer<LumaA<u8>, Vec<u8>> {
    let img_gs = img.into_luma_alpha8();
    let i_shades = find_color_regions(n_read_colors, img_gs);

    ImageBuffer::new(1, 1)
}

/// Finds the indices of the regions that each pixel belongs to.
/// Identification is based on the spectrum of shades determined by the
/// number of colors input.
fn find_color_regions(n_read_colors: u8, img_gs: ImageBuffer<LumaA<u8>, Vec<u8>>) -> Vec<u8> {
    // Gets vector of shades indices for each pixel
    let step = std::u8::MAX / n_read_colors;
    let mut i_shades: Vec<u8> = Vec::new();
    for pixel in img_gs.pixels() {
        let i_shade: u8 = pixel.channels()[0] / step;
        i_shades.push(i_shade);
    }

    i_shades
}
