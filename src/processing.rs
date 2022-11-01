use image::{DynamicImage, ImageBuffer, LumaA, Pixel};
use std::{cmp, fmt::Error};

/// Processes an image.
pub fn process_img(
    img: DynamicImage,
    n_shades: u8,
    n_grad_dir: u32,
) -> ImageBuffer<LumaA<u8>, Vec<u8>> {
    let img_gs = img.into_luma_alpha8();
    let i_shades = find_color_regions(n_shades, &img_gs);
    let grad_dirs = gen_min_grad_map(&img_gs, n_grad_dir);

    ImageBuffer::new(1, 1)
}

/// Finds the indices of the regions that each pixel belongs to. Identification is based on the
/// spectrum of shades determined by the number of colors input.
fn find_color_regions(n_shades: u8, img_gs: &ImageBuffer<LumaA<u8>, Vec<u8>>) -> Vec<u8> {
    // Gets vector of shades indices for each pixel
    let step = std::u8::MAX / n_shades;
    let mut i_shades: Vec<u8> = Vec::new();
    for pixel in img_gs.pixels() {
        let i_shade: u8 = pixel.channels()[0] / step;
        i_shades.push(i_shade);
    }

    i_shades
}

/// Generates the minimum shade gradient map for a grayscale image. This map contains the directions
/// to where the shade changes less for each pixel, relative to the other analyzed directions.
fn gen_min_grad_map(img_gs: &ImageBuffer<LumaA<u8>, Vec<u8>>, n_grad_dir: u32) -> Vec<u16> {
    // Finds radius of gradient analysis
    let n_pixel_layers = n_grad_dir / 4 + 1;
    println!("n_pixel_layers: {:?}", n_pixel_layers);

    // Finds the direction of minimum gradient for each pixel
    let x_max = img_gs.width() - 1;
    let y_max = img_gs.height() - 1;
    let mut dirs: Vec<u16> = Vec::new();
    for (x, y, _) in img_gs.enumerate_pixels() {
        // Calculates coordinates of pixel range to use in gradient calculation
        let x_beg = cmp::max(x as i32 - n_pixel_layers as i32, 0) as u32;
        let x_end = cmp::min(x + n_pixel_layers as u32, x_max);
        let y_beg = cmp::max(y as i32 - n_pixel_layers as i32, 0) as u32;
        let y_end = cmp::min(y + n_pixel_layers as u32, y_max);
        // Gets the pixel range
        let mut pixels: Vec<&LumaA<u8>> = Vec::new();
        for x in x_beg..=x_end {
            for y in y_beg..=y_end {
                pixels.push(img_gs.get_pixel(x, y));
            }
        }
        // Determines current pixel's index in pixel range vector
        let pixels_len = pixels.len() as u32;
        let i_pixel = det_i_pixel(x, y, x_max, y_max, x_beg, x_end, y_beg, y_end, pixels_len);
        // Finds direction of minimal shade gradient
        dirs.push(find_min_grad_dir(pixels, i_pixel));
    }

    dirs
}

/// Determines the index of the reference pixel in a vector of pixels for the gradient computation.
fn det_i_pixel(
    x: u32,
    y: u32,
    x_max: u32,
    y_max: u32,
    x_beg: u32,
    x_end: u32,
    y_beg: u32,
    y_end: u32,
    pixels_len: u32,
) -> u32 {
    if x == 0 && y == 0 {
        0 // First element
    } else if x == x_max && y == 0 {
        pixels_len - (x_end - x_beg) // First pixel of last row
    } else if x == 0 && y == y_max {
        x_end - x_beg // Last pixel of first row
    } else if x == x_max && y == y_max {
        pixels_len - 1
    } else if x == 0 {
        y - y_beg // yth pixel of first row
    } else if y == 0 {
        (1 + x_end - x_beg) * (y_end - y_beg) / 2 // First column of mid row
    } else if x == x_max {
        (pixels_len + x_end - x_beg) / 2 // Last pixel of mid row
    } else if y == y_max {
        pixels_len - 1 - (x_end - x_beg) / 2 // Mid column of last row
    } else {
        pixels_len / 2 // Central pixel
    }
}

fn find_min_grad_dir(pixel_range: Vec<&LumaA<u8>>, i_pixel: u32) -> u16 {
    0
}

fn calc_grad(img_gs: ImageBuffer<LumaA<u8>, Vec<u8>>, i_shades: Vec<u8>, dir: u8) -> f32 {
    0.0
}
