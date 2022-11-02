use image::{DynamicImage, ImageBuffer, LumaA, Pixel};
use std::{cmp, f32::consts::PI};

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
fn gen_min_grad_map(img_gs: &ImageBuffer<LumaA<u8>, Vec<u8>>, n_grad_dir: u32) -> Vec<f32> {
    // Finds radius of gradient analysis
    let n_pixel_layers = n_grad_dir / 4 + 1;
    // Generates test directions vector
    let mut test_directs: Vec<f32> = Vec::new();
    for i_dir in 0..n_grad_dir {
        test_directs.push(i_dir as f32 * PI / n_grad_dir as f32);
    }

    // Finds the direction of minimum gradient for each pixel
    let x_max = img_gs.width() - 1;
    let y_max = img_gs.height() - 1;
    let mut min_grad_directs_map: Vec<f32> = Vec::new();
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
        min_grad_directs_map.push(find_min_grad_dir(
            pixels,
            ij_pixel,
            &test_directs,
            n_pixel_layers,
        ));
    }

    min_grad_directs_map
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

fn find_min_grad_dir(
    pixels: Vec<Vec<&LumaA<u8>>>,
    ij_pixel: (u32, u32),
    test_directs: &Vec<f32>,
    n_pixel_layers: u32,
) -> f32 {
    // Finds pixels that compose gradient
    let mut angles: Vec<Vec<Option<f32>>> = Vec::new();
    for i in 0..pixels.len() {
        angles.push(Vec::new());
        for j in 0..pixels[i].len() {
            // Prevents invalid operations
            if (i as u32, j as u32) == ij_pixel {
                angles[i].push(None);
                continue;
            }
            // Finds relative angle for current pixel
            let i_delta = i as u32 - ij_pixel.0;
            let j_delta = j as u32 - ij_pixel.1;
            let angle = (i_delta as f32).atan2(j_delta as f32);
            angles[i].push(Some(angle));
        }
    }
    let mut grads: Vec<f32> = Vec::new();
    // Finds the gradients in every test direction
    for direct in test_directs {
        grads.push(calc_grad(&pixels, direct, &n_pixel_layers));
    }

    grads[0]
}

fn calc_grad(pixels: &Vec<Vec<&LumaA<u8>>>, direct: &f32, n_pixel_layers: &u32) -> f32 {
    // Selects pixels that compose gradient of the given direction
    let ij_pixels = get_pixels_in_line(direct, n_pixel_layers);
    // Computes
    0.0
}

/// Gets a vector of pixel coordinates that are in the directions specified. Reference is a central
/// pixel of the pixel matrix of n_pixel_layers surrounding it.
fn get_pixels_in_line(direct: &f32, n_pixel_layers: &u32) -> Vec<(u32, u32)> {
    let mut ij_pixels: Vec<(u32, u32)> = Vec::new();
    for i_layer in 1..*n_pixel_layers {
        // Avoids division by 0 (and small numbers)
        let ij = if direct < &(PI / 4.0) || direct > &(PI * 3.0 / 4.0) {
            // Multiplies using the tangent value directly (between -1 and 1)
            let i = i_layer;
            let j = (i_layer as f32 * direct.tan()).round() as u32;
            (i, j)
        } else {
            // Multiplies using the cotangent value (cos/sin) (between -1 and 1)
            let i = (i_layer as f32 * direct.cos() / direct.sin()).round() as u32;
            let j = i_layer;
            (i, j)
        };
        ij_pixels.push(ij);
    }

    ij_pixels
}
