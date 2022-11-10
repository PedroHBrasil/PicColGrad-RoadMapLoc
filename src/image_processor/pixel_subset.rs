use std::{
    f64::{consts::PI, INFINITY},
    fmt::Error,
};

use image::{GrayAlphaImage, LumaA};

/// Represents a subset of pixels of a GrayAlphaImage.
pub struct PixelSubset<'a> {
    /// Source image for the pixels
    src_img: &'a GrayAlphaImage,
    /// Pixels that compose the subset (references to original pixels)
    pixels: Vec<Vec<Option<&'a LumaA<u8>>>>,
    // Number of pixel layers that compose the subset
    n_layers: u32,
}

impl PixelSubset<'_> {
    /// Initializes a new PixelSubset
    pub fn new<'a>(img: &'a GrayAlphaImage, n_layers: u32) -> PixelSubset<'a> {
        PixelSubset {
            src_img: img,
            pixels: Vec::with_capacity(((2 * n_layers + 1) as usize).pow(2)),
            n_layers,
        }
    }

    /// Fills the PixelSubset's pixels with the pixel references to the referenced image.
    pub fn fill(&mut self, ref_pixel_coords: (u32, u32)) -> Result<usize, Error> {
        // Gets valid coordinate limits
        let x_max = self.src_img.width() as i32 - 1;
        let y_max = self.src_img.height() as i32 - 1;
        // Calculates coordinates of pixel range to use in gradient calculation
        let x_beg = ref_pixel_coords.0 as i32 - self.n_layers as i32;
        let x_end = ref_pixel_coords.0 as i32 + self.n_layers as i32;
        let y_beg = ref_pixel_coords.1 as i32 - self.n_layers as i32;
        let y_end = ref_pixel_coords.1 as i32 + self.n_layers as i32;
        // Gets the pixel range
        for x_i in x_beg..=x_end {
            self.pixels.push(Vec::new());
            for y_i in y_beg..=y_end {
                if x_i < 0 || x_i > x_max || y_i < 0 || y_i > y_max {
                    self.pixels.last_mut().unwrap().push(None);
                } else {
                    let pixel = self.src_img.get_pixel(x_i as u32, y_i as u32);
                    self.pixels.last_mut().unwrap().push(Some(pixel));
                }
            }
        }

        Ok(self.pixels.len())
    }

    /// Finds the index of the direction with the lowest color gradient value for a given pixel set.
    pub fn find_i_min_grad_dir(&self, test_directs: &Vec<f64>) -> usize {
        // Finds the gradients in every test direction
        let mut grads = Vec::new();
        for direct in test_directs {
            grads.push(self.calc_grad(*direct));
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
    fn calc_grad(&self, direct: f64) -> f64 {
        // Selects pixels that compose the gradient of the given direction
        let ij_pixels = self.get_pixels_in_line(direct);
        let n_pxl = ij_pixels.len() as f64;
        // Gets the gradients between the pixels
        let mut pxl_grads = Vec::new();
        for idx in 1..ij_pixels.len() {
            // Gets indices
            let i = ij_pixels[idx].0 as f64;
            let j = ij_pixels[idx].1 as f64;
            let i_prev = ij_pixels[idx - 1].0 as f64;
            let j_prev = ij_pixels[idx - 1].1 as f64;
            // Calculates distance
            let dist = ((i - i_prev).powf(2.0) + (j - j_prev).powf(2.0)).powf(0.5);
            // Gets shade values
            let shade = self.pixels[i as usize][j as usize].unwrap().0[0] as f64;
            let shade_prev = self.pixels[i_prev as usize][j_prev as usize].unwrap().0[0] as f64;
            // Calculates differential
            let grad = (shade - shade_prev) / dist;

            pxl_grads.push(grad);
        }

        // Calculates the pixel line's average gradient
        let grad = pxl_grads
            .into_iter()
            .reduce(|accum, grad_local| accum + grad_local / (n_pxl - 1.0))
            .unwrap_or_else(|| INFINITY)
            .abs();

        grad
    }

    /// Gets a vector of pixel coordinates that are in the directions specified. Reference is a central
    /// pixel of the pixel matrix of n_pixel_layers surrounding it.
    fn get_pixels_in_line(&self, direct: f64) -> Vec<(u32, u32)> {
        let mut ij_pixels: Vec<(u32, u32)> = Vec::new();
        // Coordinates of central (reference) subpixel
        let ij_center = self.n_layers as i32;
        // Going from -n_layers to +n_layers inclusive allows for sorted insertion in vec
        for i_layer in -(self.n_layers as i32)..=(self.n_layers as i32) {
            // Avoids division by 0 (and small numbers)
            let (i, j) = if direct <= (PI / 4.0) || direct > (PI * 3.0 / 4.0) {
                // Multiplies using the tangent value directly (between -1 and 1)
                let i = ij_center - (i_layer as f64 * direct.tan()).round() as i32;
                let j = ij_center + i_layer;
                (i, j)
            } else {
                // Multiplies using the cotangent value (cos/sin) (between -1 and 1)
                let i = ij_center + i_layer;
                let j = ij_center - (i_layer as f64 * direct.cos() / direct.sin()).round() as i32;
                (i, j)
            };
            // Checks if coordinate is valid before pushing it
            if let Some(_) = self.pixels[i as usize][j as usize] {
                ij_pixels.push((i as u32, j as u32));
            }
        }

        ij_pixels
    }
}

#[cfg(test)]
mod tests {

    use crate::test_util;

    use super::*;

    #[test]
    fn find_i_min_grad_img_5x5_dir_0_ij_2() {
        let x = 2;
        let y = 2;

        let img_gs: GrayAlphaImage = test_util::tests::img_grad_factory(5, 5, 0.0);
        let mut pxl_subset = PixelSubset::new(&img_gs, 1);
        pxl_subset.fill((x, y)).unwrap();

        let expected = 0;
        let result = pxl_subset.find_i_min_grad_dir(&vec![0.0, PI / 4.0, PI / 2.0, 3.0 * PI / 4.0]);

        assert_eq!(expected, result);
    }

    #[test]
    fn find_i_min_grad_img_5x5_dir_0_ij_0() {
        let x = 0;
        let y = 0;

        let img_gs: GrayAlphaImage = test_util::tests::img_grad_factory(5, 5, 0.0);
        let mut pxl_subset = PixelSubset::new(&img_gs, 1);
        pxl_subset.fill((x, y)).unwrap();

        let expected = 0;
        let result = pxl_subset.find_i_min_grad_dir(&vec![0.0, PI / 4.0, PI / 2.0, 3.0 * PI / 4.0]);

        assert_eq!(expected, result);
    }

    #[test]
    fn calc_grad_img_5x5_dir_0_ij_0() {
        let x = 2;
        let y = 2;

        let img_gs: GrayAlphaImage = test_util::tests::img_grad_factory(5, 5, 0.0);
        let mut pxl_subset = PixelSubset::new(&img_gs, 1);
        pxl_subset.fill((x, y)).unwrap();

        let expected = 0.0;
        let result = pxl_subset.calc_grad(0.0);

        assert_eq!(expected, result);
    }

    #[test]
    fn calc_grad_img_5x5_dir_0_ij_2() {
        let x = 2;
        let y = 2;

        let img_gs: GrayAlphaImage = test_util::tests::img_grad_factory(5, 5, 0.0);
        let mut pxl_subset = PixelSubset::new(&img_gs, 1);
        pxl_subset.fill((x, y)).unwrap();

        let expected = 0.0;
        let result = pxl_subset.calc_grad(0.0);

        assert_eq!(expected, result);
    }

    #[test]
    fn calc_grad_img_5x5_dir_pi4_ij_0() {
        let x = 0;
        let y = 0;

        let img_gs: GrayAlphaImage = test_util::tests::img_grad_factory(5, 5, PI / 4.0);
        let mut pxl_subset = PixelSubset::new(&img_gs, 1);
        pxl_subset.fill((x, y)).unwrap();

        let expected = INFINITY;
        let result = pxl_subset.calc_grad(PI / 4.0);

        assert_eq!(expected, result);
    }

    #[test]
    fn calc_grad_img_5x5_dir_pi4_ij_2() {
        let x = 2;
        let y = 2;

        let img_gs: GrayAlphaImage = test_util::tests::img_grad_factory(5, 5, PI / 4.0);
        let mut pxl_subset = PixelSubset::new(&img_gs, 1);
        pxl_subset.fill((x, y)).unwrap();

        let expected = 0.0;
        let result = pxl_subset.calc_grad(PI / 4.0);

        assert_eq!(expected, result);
    }

    #[test]
    fn calc_grad_img_5x5_dir_pi4_ij_4() {
        let x = 4;
        let y = 4;

        let img_gs: GrayAlphaImage = test_util::tests::img_grad_factory(5, 5, PI / 4.0);
        let mut pxl_subset = PixelSubset::new(&img_gs, 1);
        pxl_subset.fill((x, y)).unwrap();

        let expected = INFINITY;
        let result = pxl_subset.calc_grad(PI / 4.0);

        assert_eq!(expected, result);
    }

    #[test]
    fn fill_5x5_x0_y0_n1() {
        let x = 0;
        let y = 0;

        let img_gs: GrayAlphaImage = test_util::tests::img_grad_factory(5, 5, 3.0 * PI / 4.0);
        let mut pxl_subset = PixelSubset::new(&img_gs, 1);
        pxl_subset.fill((x, y)).unwrap();

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
        let result = pxl_subset.pixels;

        assert_eq!(expected, result);
    }

    #[test]
    fn fill_5x5_x2_y2_n1() {
        let x = 2;
        let y = 2;

        let img_gs: GrayAlphaImage = test_util::tests::img_grad_factory(5, 5, 3.0 * PI / 4.0);
        let mut pxl_subset = PixelSubset::new(&img_gs, 1);
        pxl_subset.fill((x, y)).unwrap();

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
        let result = pxl_subset.pixels;

        assert_eq!(expected, result);
    }

    #[test]
    fn get_pixels_in_line_dir_0() {
        let img_gs: GrayAlphaImage = test_util::tests::img_grad_factory(5, 5, 3.0 * PI / 4.0);
        let direct = 0.0;
        let n_layers = 2;
        let mut pxl_subset = PixelSubset::new(&img_gs, n_layers);
        pxl_subset.fill((2, 2)).unwrap();

        let expected: Vec<(u32, u32)> = vec![(2, 0), (2, 1), (2, 2), (2, 3), (2, 4)];
        let result = pxl_subset.get_pixels_in_line(direct);

        assert_eq!(expected, result);
    }

    #[test]
    fn get_pixels_in_line_dir_pi4() {
        let img_gs: GrayAlphaImage = test_util::tests::img_grad_factory(5, 5, 3.0 * PI / 4.0);
        let direct = PI / 4.0;
        let n_layers = 2;
        let mut pxl_subset = PixelSubset::new(&img_gs, n_layers);
        pxl_subset.fill((2, 2)).unwrap();

        let expected: Vec<(u32, u32)> = vec![(4, 0), (3, 1), (2, 2), (1, 3), (0, 4)];
        let result = pxl_subset.get_pixels_in_line(direct);

        assert_eq!(expected, result);
    }

    #[test]
    fn get_pixels_in_line_dir_pi2() {
        let img_gs: GrayAlphaImage = test_util::tests::img_grad_factory(5, 5, 3.0 * PI / 4.0);
        let direct = PI / 2.0;
        let n_layers = 2;
        let mut pxl_subset = PixelSubset::new(&img_gs, n_layers);
        pxl_subset.fill((2, 2)).unwrap();

        let expected: Vec<(u32, u32)> = vec![(0, 2), (1, 2), (2, 2), (3, 2), (4, 2)];
        let result = pxl_subset.get_pixels_in_line(direct);

        assert_eq!(expected, result);
    }

    #[test]
    fn get_pixels_in_line_dir_pi34() {
        let img_gs: GrayAlphaImage = test_util::tests::img_grad_factory(5, 5, 3.0 * PI / 4.0);
        let direct = 3.0 * PI / 4.0;
        let n_layers = 2;
        let mut pxl_subset = PixelSubset::new(&img_gs, n_layers);
        pxl_subset.fill((2, 2)).unwrap();

        let expected: Vec<(u32, u32)> = vec![(0, 0), (1, 1), (2, 2), (3, 3), (4, 4)];
        let result = pxl_subset.get_pixels_in_line(direct);

        assert_eq!(expected, result);
    }

    #[test]
    fn get_pixels_in_line_dir_pi() {
        let img_gs: GrayAlphaImage = test_util::tests::img_grad_factory(5, 5, 3.0 * PI / 4.0);
        let direct = PI;
        let n_layers = 2;
        let mut pxl_subset = PixelSubset::new(&img_gs, n_layers);
        pxl_subset.fill((2, 2)).unwrap();

        let expected: Vec<(u32, u32)> = vec![(2, 0), (2, 1), (2, 2), (2, 3), (2, 4)];
        let result = pxl_subset.get_pixels_in_line(direct);

        assert_eq!(expected, result);
    }
}
