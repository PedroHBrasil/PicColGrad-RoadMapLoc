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
        let mut pixels: Vec<Vec<&LumaA<u8>>> = Vec::new();
        for x in x_beg..=x_end {
            pixels.push(Vec::new());
            for y in y_beg..=y_end {
                pixels.last_mut().unwrap().push(img_gs.get_pixel(x, y));
            }
        }
        // Determines current pixel's index in pixel range vector
        let n_x = x_end - x_beg + 1;
        let n_y = y_end - y_beg + 1;
        let ij_pixel = det_ij_pixel(x, y, x_max, y_max, n_x, n_y);
        // Finds direction of minimal shade gradient
        dirs.push(find_min_grad_dir(pixels, ij_pixel));
    }

    dirs
}

/// Determines the index of the reference pixel in a vector of pixels for the gradient computation.
fn det_ij_pixel(x: u32, y: u32, x_max: u32, y_max: u32, n_x: u32, n_y: u32) -> (u32, u32) {
    if x == 0 && y == 0 {
        (0, 0) // First pixel
    } else if x == x_max && y == 0 {
        (n_x - 1, 0) // First pixel of last row
    } else if x == 0 && y == y_max {
        (0, n_y - 1) // Last pixel of first row
    } else if x == x_max && y == y_max {
        (n_x - 1, n_y - 1) // Last pixel of last row
    } else if x == 0 {
        (0, n_y / 2) // Mid pixel of first row
    } else if y == 0 {
        (n_x / 2, 0) // First pixel of mid row
    } else if x == x_max {
        (n_x / 2, n_y - 1) // Last pixel of mid row
    } else if y == y_max {
        (n_x - 1, n_y / 2) // Mid column of last row
    } else {
        (n_x / 2, n_y / 2) // Central pixel
    }
}

fn find_min_grad_dir(pixels: Vec<Vec<&LumaA<u8>>>, ij_pixel: (u32, u32)) -> u16 {
    0
}

fn calc_grad(img_gs: ImageBuffer<LumaA<u8>, Vec<u8>>, i_shades: Vec<u8>, dir: u8) -> f32 {
    0.0
}
