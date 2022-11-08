use image::{DynamicImage, GrayAlphaImage, ImageBuffer, LumaA, Pixel};
use indicatif::ProgressBar;
use std::f64::consts::PI;

/// Processes an image.
pub fn process_img(
    img: DynamicImage,
    n_shades: u8,
    n_grad_dir: u32,
) -> ImageBuffer<LumaA<u8>, Vec<u8>> {
    let img_gs = img.into_luma_alpha8();
    let i_shades = find_color_regions(n_shades, &img_gs);
    let min_grad_dir_map = gen_min_grad_map(&img_gs, n_grad_dir);
    //TODO Make straight lines filter function.

    ImageBuffer::new(1, 1)
}

/// Represents a shade region of a grayscale image.
#[derive(PartialEq, Debug)]
struct ShadeRegion {
    // Coordinates of the pixels contained in the region
    coords: Vec<(u32, u32)>,
    // Shade index (0 to n_shades-1)
    i_shade: u8,
}

impl ShadeRegion {
    // Finds the pixels that belong to this region
    fn find_all_coords(
        &mut self,
        img_gs: &GrayAlphaImage,
        i_shades: &Vec<Vec<u8>>,
        alloc_pixels: &mut Vec<Vec<bool>>,
    ) {
        let mut n_coords_to_check = 1;
        while n_coords_to_check > 0 {
            // Index of first coordinate to look for neighbors
            let i_first_coord = self.coords.len() - n_coords_to_check;
            // Finds neighbors to push to this region
            let mut coords_to_push: Vec<(u32, u32)> =
                Vec::with_capacity(8 * (self.coords.len() - i_first_coord));
            for ref_coords in self.coords[i_first_coord..self.coords.len()].iter() {
                let mut neighbors_to_append =
                    self.find_neighbors(ref_coords, img_gs, i_shades, alloc_pixels);
                coords_to_push.append(&mut neighbors_to_append);
            }
            // Updates number of coordinates to check
            n_coords_to_check = coords_to_push.len();
            // Pushes new coords to region
            for new_coords in coords_to_push.into_iter() {
                self.coords.push(new_coords);
                alloc_pixels[new_coords.0 as usize][new_coords.1 as usize] = true;
            }
        }
        // Removes unused space
        self.coords.shrink_to_fit();
    }

    /// Adds the reference pixel's neighbors that have the same shade index
    fn find_neighbors(
        &self,
        ref_coords: &(u32, u32),
        img_gs: &GrayAlphaImage,
        i_shades: &Vec<Vec<u8>>,
        alloc_pixels: &mut Vec<Vec<bool>>,
    ) -> Vec<(u32, u32)> {
        // Checks and adds horizontal and vertical neighbors
        let mut coords_to_add = Vec::with_capacity(8);
        for coord_delta in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
            let x = ref_coords.0 as i32 + coord_delta.0;
            let y = ref_coords.1 as i32 + coord_delta.1;
            // Skips if pixel coordinates is out of bounds
            if x < 0 || y < 0 || x >= img_gs.width() as i32 || y >= img_gs.height() as i32 {
                continue;
            }
            // Checks if the shade of the current neighbor is equal to the region and if the
            // current pixel has already been assigned to a region.
            if i_shades[x as usize][y as usize] == self.i_shade
                && !alloc_pixels[x as usize][y as usize]
            {
                // Assigns the pixel's coordinates to the region
                coords_to_add.push((x as u32, y as u32));
                alloc_pixels[x as usize][y as usize] = true;
            }
        }

        coords_to_add
    }
}

/// Finds the indices of the regions that each pixel belongs to. Identification is based on the
/// spectrum of shades determined by the number of colors input.
fn find_color_regions(n_shades: u8, img_gs: &GrayAlphaImage) -> Vec<ShadeRegion> {
    // Gets the vector of shade indexes based on an equaly spaced distribution of shade resolution
    // reduction
    let i_shades = det_i_shades(n_shades, img_gs);
    // Creates and finds the regions for each cluster of pixels with equal shade index
    let mut alloc_pixels = vec![vec![false; img_gs.height() as usize]; img_gs.width() as usize];
    let mut shade_regions = Vec::new();
    let pb = ProgressBar::new(img_gs.pixels().len() as u64);
    println!("Making shade regions");
    for (x, y, _) in pb.wrap_iter(img_gs.enumerate_pixels()) {
        // Checks if the pixel has already been allocated
        let allocated = alloc_pixels[x as usize][y as usize];
        if allocated {
            continue;
        }
        // Initializes a new region
        let i_shade: u8 = i_shades[x as usize][y as usize];
        let n_to_alloc = img_gs.pixels().len()
            - alloc_pixels
                .iter()
                .flatten()
                .filter(|allocated| **allocated)
                .count();
        let mut shade_region = ShadeRegion {
            coords: Vec::with_capacity(2 * n_to_alloc),
            i_shade,
        };
        // Adds first pixel
        shade_region.coords.push((x, y));
        alloc_pixels[x as usize][y as usize] = true;
        // Adds remaining pixels of the region
        shade_region.find_all_coords(img_gs, &i_shades, &mut alloc_pixels);
        // Adds region to regions vec
        shade_regions.push(shade_region);
    }

    shade_regions
}

/// Gets the vector of shade indexes based on an equaly spaced distribution of shade resolution
/// reduction
fn det_i_shades(n_shades: u8, img_gs: &GrayAlphaImage) -> Vec<Vec<u8>> {
    let step = std::u8::MAX / (n_shades - 1);
    let mut i_shades = vec![vec![0 as u8; img_gs.height() as usize]; img_gs.width() as usize];
    for x in 0..img_gs.width() {
        for y in 0..img_gs.height() {
            let mut i_shade = img_gs.get_pixel(x, y)[0] / step;
            if i_shade == n_shades {
                i_shade -= 1
            }
            i_shades[x as usize][y as usize] = i_shade;
        }
    }

    i_shades
}

/// Generates the minimum shade gradient map for a grayscale image. This map contains the directions
/// to where the shade changes less for each pixel, relative to the other analyzed directions.
fn gen_min_grad_map(img_gs: &GrayAlphaImage, n_grad_dir: u32) -> Vec<Vec<f64>> {
    // Finds radius of gradient analysis
    let n_pixel_layers = n_grad_dir / 4 + 1;
    // Generates test directions vector
    let mut test_directs = Vec::new();
    for i_dir in 0..n_grad_dir {
        test_directs.push(i_dir as f64 * PI / n_grad_dir as f64);
    }
    // Finds the direction of minimum gradient for each pixel
    let mut min_grad_directs_map =
        vec![vec![0.0; img_gs.height() as usize]; img_gs.width() as usize];
    for (x, y, _) in img_gs.enumerate_pixels() {
        let subpixels = get_subpixels(img_gs, x, y, n_pixel_layers);
        // Finds direction of minimal shade gradient
        let i_min_grad_dir = find_i_min_grad_dir(subpixels, &test_directs, n_pixel_layers);
        min_grad_directs_map[x as usize][y as usize] = test_directs[i_min_grad_dir as usize];
    }

    min_grad_directs_map
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
) -> usize {
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
    let i_min_grad = grads.iter().position(|grad| *grad == *min_grad).unwrap();

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

/// Makes the straight line image based on the grayscale image, the shades regions and the minimal
/// gradient directions map.
fn make_straight_lines_img(
    img_gs: &GrayAlphaImage,
    i_shades: Vec<u8>,
    n_shades: u8,
    grad_dirs: Vec<f64>,
    n_grad_dir: u32,
) -> GrayAlphaImage {
    GrayAlphaImage::new(1, 1)
}

/// Determines the shade of a pixel of the straight lines filter image based on the region's shade
/// and the average gradient direction
fn det_straight_line_pxl_shade(x: u32, y: u32, i_shade: u8, n_shades: u8, avg_grad_dir: f64) -> u8 {
    //
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn img_grad_factory(width: u32, height: u32, grad_dir: f64) -> GrayAlphaImage {
        // Finds ratios and virtual image sizes
        let x_ratio = grad_dir.cos().abs() / (grad_dir.cos().abs() + grad_dir.sin().abs());
        let y_ratio = grad_dir.sin().abs() / (grad_dir.cos().abs() + grad_dir.sin().abs());
        let (width_virt, height_virt) =
            ((width - 1) as f64 * x_ratio, (height - 1) as f64 * y_ratio);
        // Makes image buffer
        let img = ImageBuffer::from_fn(width, height, |x, y| -> LumaA<u8> {
            // Makes pixel according to the gradient direction pattern
            let (x_virt, y_virt) = (x as f64 * x_ratio, y as f64 * y_ratio);
            let ratio = (x_virt + y_virt) / (width_virt + height_virt);
            let shade = (std::u8::MAX as f64 * ratio) as u8;
            LumaA([shade, std::u8::MAX])
        });

        img
    }

    #[test]
    fn shade_region_find_neighbors() {
        let img_gs = img_grad_factory(3, 3, 0.0);

        let shade_region = ShadeRegion {
            coords: vec![(0, 0)],
            i_shade: 0,
        };

        let i_shades = det_i_shades(3, &img_gs);
        let mut alloc_pixels = vec![vec![false; img_gs.height() as usize]; img_gs.width() as usize];

        for shade_vec in &i_shades {
            println!("i_shades_col: {:?}", shade_vec);
        }

        let expected = vec![(0, 0), (0, 2)];
        let result = shade_region.find_neighbors(&(0, 1), &img_gs, &i_shades, &mut alloc_pixels);

        assert_eq!(expected, result);
    }

    #[test]
    fn shade_region_find_all_coords() {
        let img_gs = img_grad_factory(5, 5, 0.0);

        let i_shades = det_i_shades(5, &img_gs);
        let mut alloc_pixels = vec![vec![false; img_gs.height() as usize]; img_gs.width() as usize];

        let mut shade_region = ShadeRegion {
            coords: vec![(0, 0)],
            i_shade: 0,
        };

        alloc_pixels[0][0] = true;

        for shade_vec in &i_shades {
            println!("i_shades_col: {:?}", shade_vec);
        }
        shade_region.find_all_coords(&img_gs, &i_shades, &mut alloc_pixels);

        let expected = vec![(0, 0), (0, 1), (0, 2), (0, 3), (0, 4)];
        let result = shade_region.coords;

        assert_eq!(expected, result);
    }

    #[test]
    fn find_color_regions_5x5_dir0() {
        let img_gs = img_grad_factory(5, 5, 0.0);
        let n_shades = 5;

        let expected = vec![
            ShadeRegion {
                coords: vec![(0, 0), (0, 1), (0, 2), (0, 3), (0, 4)],
                i_shade: 0,
            },
            ShadeRegion {
                coords: vec![(1, 0), (1, 1), (1, 2), (1, 3), (1, 4)],
                i_shade: 1,
            },
            ShadeRegion {
                coords: vec![(2, 0), (2, 1), (2, 2), (2, 3), (2, 4)],
                i_shade: 2,
            },
            ShadeRegion {
                coords: vec![(3, 0), (3, 1), (3, 2), (3, 3), (3, 4)],
                i_shade: 3,
            },
            ShadeRegion {
                coords: vec![(4, 0), (4, 1), (4, 2), (4, 3), (4, 4)],
                i_shade: 4,
            },
        ];
        let result = find_color_regions(n_shades, &img_gs);

        assert_eq!(expected, result);
    }

    #[test]
    fn get_subpixels_5x5_x0_y0_n1() {
        let x = 0;
        let y = 0;

        let img_gs: GrayAlphaImage = img_grad_factory(5, 5, 3.0 * PI / 4.0);

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

        let img_gs: GrayAlphaImage = img_grad_factory(5, 5, 3.0 * PI / 4.0);

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
