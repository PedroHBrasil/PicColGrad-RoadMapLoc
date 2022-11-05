use image::{DynamicImage, GrayAlphaImage, ImageBuffer, LumaA, Pixel};
use std::{cmp, f64::consts::PI};

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
fn find_color_regions(n_shades: u8, img_gs: &GrayAlphaImage) -> Vec<u8> {
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
fn gen_min_grad_map(img_gs: &GrayAlphaImage, n_grad_dir: u32) -> Vec<u32> {
    // Finds radius of gradient analysis
    let n_pixel_layers = n_grad_dir / 4 + 1;
    // Generates test directions vector
    let mut test_directs = Vec::new();
    for i_dir in 0..n_grad_dir {
        test_directs.push(i_dir as f64 * PI / n_grad_dir as f64);
    }
    // Finds the direction of minimum gradient for each pixel
    let mut min_i_grad_directs_map: Vec<u32> = Vec::new();
    for (x, y, _) in img_gs.enumerate_pixels() {
        let subpixels = get_subpixels(img_gs, x, y, n_pixel_layers);
        // Finds direction of minimal shade gradient
        let i_min_grad_dir = find_i_min_grad_dir(subpixels, &test_directs, n_pixel_layers);
        min_i_grad_directs_map.push(i_min_grad_dir);
    }

    min_i_grad_directs_map
}

/// Gets the matrix of subpixels based on the coordinates of the central pixel and the number of
/// layers.
fn get_subpixels(
    img_gs: &GrayAlphaImage,
    x: u32,
    y: u32,
    n_pixel_layers: u32,
) -> Vec<Vec<Option<&LumaA<u8>>>> {
    // Gets valid coordinate limits
    let x_max = img_gs.width() as i32 - 1;
    let y_max = img_gs.height() as i32 - 1;
    // Calculates coordinates of pixel range to use in gradient calculation
    let x_beg = x as i32 - n_pixel_layers as i32;
    let x_end = x as i32 + n_pixel_layers as i32;
    let y_beg = y as i32 - n_pixel_layers as i32;
    let y_end = y as i32 + n_pixel_layers as i32;
    // Gets the pixel range
    let mut subpixels: Vec<Vec<Option<&LumaA<u8>>>> = Vec::new();
    for x_i in x_beg..=x_end {
        subpixels.push(Vec::new());
        for y_i in y_beg..=y_end {
            if x_i < 0 || x_i > x_max || y_i < 0 || y_i > y_max {
                subpixels.last_mut().unwrap().push(None);
            } else {
                let pixel = img_gs.get_pixel(x_i as u32, y_i as u32);
                subpixels.last_mut().unwrap().push(Some(pixel));
            }
        }
    }

    subpixels
}

/// Finds the index of the direction with the lowest color gradient value for a given pixel set.
fn find_i_min_grad_dir(
    subpixels: Vec<Vec<Option<&LumaA<u8>>>>,
    test_directs: &Vec<f64>,
    n_pixel_layers: u32,
) -> u32 {
    // Finds the gradients in every test direction
    let mut grads = Vec::new();
    for direct in test_directs {
        grads.push(calc_grad(&subpixels, *direct, n_pixel_layers));
    }
    // Gets index of direction of minimal gradient
    let min_grad = grads
        .iter()
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();
    let i_min_grad = grads.iter().position(|grad| *grad == *min_grad).unwrap() as u32;

    i_min_grad
}

/// Computes the color gradient for a pixel set in a given direction.
fn calc_grad(subpixels: &Vec<Vec<Option<&LumaA<u8>>>>, direct: f64, n_pixel_layers: u32) -> f64 {
    // Selects pixels that compose the gradient of the given direction
    let ij_pixels = get_pixels_in_line(direct, n_pixel_layers as i32);
    // Computes the gradient value
    let n_subpixels = n_pixel_layers * 2 + 1;
    let mut shade_sum = 0.0;
    let mut n_shades = 0.0;
    for i_subpixel in 0..n_subpixels {
        let (i, j) = ij_pixels[i_subpixel as usize];
        let subpixel_opt = subpixels[i as usize][j as usize];
        if let Some(subpixel) = subpixel_opt {
            let shade = subpixel.0[0];
            shade_sum += shade as f64;
            n_shades += 1.0;
        }
    }
    let grad = shade_sum / n_shades;

    grad
}

/// Gets a vector of pixel coordinates that are in the directions specified. Reference is a central
/// pixel of the pixel matrix of n_pixel_layers surrounding it.
fn get_pixels_in_line(direct: f64, n_pixel_layers: i32) -> Vec<(u32, u32)> {
    let mut ij_pixels: Vec<(u32, u32)> = Vec::new();
    // Coordinates of central (reference) subpixel
    let ij_center = n_pixel_layers;
    // Going from -n_layers to +n_layers inclusive allows for sorted insertion in vec
    for i_layer in -n_pixel_layers..=n_pixel_layers {
        // Avoids division by 0 (and small numbers)
        if direct <= (PI / 4.0) || direct > (PI * 3.0 / 4.0) {
            // Multiplies using the tangent value directly (between -1 and 1)
            let i = ij_center - (i_layer as f64 * direct.tan()).round() as i32;
            let j = ij_center + i_layer;
            ij_pixels.push((i as u32, j as u32));
        } else {
            // Multiplies using the cotangent value (cos/sin) (between -1 and 1)
            let i = ij_center + i_layer;
            let j = ij_center - (i_layer as f64 * direct.cos() / direct.sin()).round() as i32;
            ij_pixels.push((i as u32, j as u32));
        }
    }

    ij_pixels
}

#[cfg(test)]
mod tests {
    use super::*;

    fn img_factory(width: u32, height: u32, grad_dir: f64) -> GrayAlphaImage {
        // Finds ratios and virtual image sizes
        println!("grad_dir: {}", grad_dir);
        println!("grad_dir_cos: {}", grad_dir.cos());
        println!("grad_dir_sin: {}", grad_dir.sin());
        let x_ratio = grad_dir.cos().abs() / (grad_dir.cos().abs() + grad_dir.sin().abs());
        let y_ratio = grad_dir.sin().abs() / (grad_dir.cos().abs() + grad_dir.sin().abs());
        let (width_virt, height_virt) =
            ((width - 1) as f64 * x_ratio, (height - 1) as f64 * y_ratio);
        println!("x_ratio: {}, y_ratio: {}", x_ratio, y_ratio);
        println!("w: {}, h: {}", width_virt, height_virt);
        // Makes image buffer
        let img = ImageBuffer::from_fn(width, height, |x, y| -> LumaA<u8> {
            // Makes pixel according to the gradient direction pattern
            let (x_virt, y_virt) = (x as f64 * x_ratio, y as f64 * y_ratio);
            println!("x: {}, y: {}", x_virt, y_virt);
            let ratio = (x_virt + y_virt) / (width_virt + height_virt);
            let shade = (std::u8::MAX as f64 * ratio) as u8;
            LumaA([shade, std::u8::MAX])
        });

        img
    }

    #[test]
    fn get_subpixels_5x5_x0_y0_n1() {
        let x = 0;
        let y = 0;

        let img_gs: GrayAlphaImage = img_factory(5, 5, 3.0 * PI / 4.0);

        let expected = vec![
            vec![None, None, None],
            vec![
                None,
                Some(img_gs.get_pixel(0, 0)),
                Some(img_gs.get_pixel(0, 1)),
            ],
            vec![
                None,
                Some(img_gs.get_pixel(1, 0)),
                Some(img_gs.get_pixel(1, 1)),
            ],
        ];
        let result = get_subpixels(&img_gs, x, y, 1);

        assert_eq!(expected, result);
    }

    #[test]
    fn get_subpixels_5x5_x2_y2_n1() {
        let x = 2;
        let y = 2;

        let img_gs: GrayAlphaImage = img_factory(5, 5, 3.0 * PI / 4.0);

        let expected = vec![
            vec![
                Some(img_gs.get_pixel(1, 1)),
                Some(img_gs.get_pixel(1, 2)),
                Some(img_gs.get_pixel(1, 3)),
            ],
            vec![
                Some(img_gs.get_pixel(2, 1)),
                Some(img_gs.get_pixel(2, 2)),
                Some(img_gs.get_pixel(2, 3)),
            ],
            vec![
                Some(img_gs.get_pixel(3, 1)),
                Some(img_gs.get_pixel(3, 2)),
                Some(img_gs.get_pixel(3, 3)),
            ],
        ];
        let result = get_subpixels(&img_gs, x, y, 1);

        assert_eq!(expected, result);
    }

    #[test]
    fn get_pixels_in_line_dir_0() {
        let direct = 0.0;
        let n_pixel_layers = 2;

        let expected: Vec<(u32, u32)> = vec![(2, 0), (2, 1), (2, 2), (2, 3), (2, 4)];
        let result = get_pixels_in_line(direct, n_pixel_layers);

        assert_eq!(expected, result);
    }

    #[test]
    fn get_pixels_in_line_dir_pi4() {
        let direct = PI / 4.0;
        let n_pixel_layers = 2;

        let expected: Vec<(u32, u32)> = vec![(4, 0), (3, 1), (2, 2), (1, 3), (0, 4)];
        let result = get_pixels_in_line(direct, n_pixel_layers);

        assert_eq!(expected, result);
    }

    #[test]
    fn get_pixels_in_line_dir_pi2() {
        let direct = PI / 2.0;
        let n_pixel_layers = 2;

        let expected: Vec<(u32, u32)> = vec![(0, 2), (1, 2), (2, 2), (3, 2), (4, 2)];
        let result = get_pixels_in_line(direct, n_pixel_layers);

        assert_eq!(expected, result);
    }

    #[test]
    fn get_pixels_in_line_dir_pi34() {
        let direct = 3.0 * PI / 4.0;
        let n_pixel_layers = 2;

        let expected: Vec<(u32, u32)> = vec![(0, 0), (1, 1), (2, 2), (3, 3), (4, 4)];
        let result = get_pixels_in_line(direct, n_pixel_layers);

        assert_eq!(expected, result);
    }

    #[test]
    fn get_pixels_in_line_dir_pi() {
        let direct = PI;
        let n_pixel_layers = 2;

        let expected: Vec<(u32, u32)> = vec![(2, 0), (2, 1), (2, 2), (2, 3), (2, 4)];
        let result = get_pixels_in_line(direct, n_pixel_layers);

        assert_eq!(expected, result);
    }
}
