use image::{DynamicImage, GrayAlphaImage};
use indicatif::ProgressBar;
use std::{f64::consts::PI, fmt::Error};
pub mod pixel_subset;
pub mod shade_region;

/// Processes an image.
pub fn run(
    img: DynamicImage,
    n_shades: u8,
    n_grad_dir: u32,
    stroke_width: u32,
) -> Result<GrayAlphaImage, Error> {
    let mut img_proc = ImageProcessor::build(img, n_shades, n_grad_dir);
    img_proc.gen_shade_regions()?;
    img_proc.calc_regions_avg_min_grad_dirs()?;
    img_proc.make_output_img(stroke_width)?;

    Ok(img_proc.img)
}

/// Contains the data needed for the image processing.
struct ImageProcessor {
    /// Grayscale version of the image to be processed.
    img: GrayAlphaImage,
    /// Number of shades to consider during processing. Pixel regions are determined based on the
    /// shades derived from this value.
    n_shades: u8,
    /// Number of directions to test the gradients for. This indirectly controls the number of
    /// pixels considered when computing a gradient, as this number is automatically set to be the
    /// minimum that results in one different gradient value for each direction considered.
    n_grad_dir: u32,
    /// Regions of pixels where the color shades and gradients are approximated to be constant.
    /// The shades determine the output image's lines width/density combinations, while the
    /// gradients determine the lines directions. These parameters are constant for each region.
    shade_regions: Vec<shade_region::ShadeRegion>,
}

impl ImageProcessor {
    /// Initializes an ImageProcessor instance
    fn build(img: DynamicImage, n_shades: u8, n_grad_dir: u32) -> ImageProcessor {
        ImageProcessor {
            img: img.into_luma_alpha8(),
            n_shades,
            n_grad_dir,
            shade_regions: Vec::new(),
        }
    }

    /// Generates the shade regions. Pixel region assigment is based on the spectrum of shades
    /// determined by the number of colors input.
    fn gen_shade_regions(&mut self) -> Result<(), Error> {
        // Gets the vector of shade indexes based on an equaly spaced distribution of shade resolution
        // reduction
        let i_shades = self.gen_i_shades();
        // Creates and finds the regions for each cluster of pixels with equal shade index
        let mut alloc_pixels =
            vec![vec![false; self.img.height() as usize]; self.img.width() as usize];
        let pb = ProgressBar::new(self.img.pixels().len() as u64);
        println!("Making shade regions...");
        for (x, y, _) in pb.wrap_iter(self.img.enumerate_pixels()) {
            // Checks if the pixel has already been allocated
            let allocated = alloc_pixels[x as usize][y as usize];
            if allocated {
                continue;
            }
            // Initializes a new region
            let i_shade: u8 = i_shades[x as usize][y as usize];
            let n_to_alloc = self.img.pixels().len()
                - alloc_pixels
                    .iter()
                    .flatten()
                    .filter(|allocated| **allocated)
                    .count();
            let mut shade_region = shade_region::ShadeRegion {
                coords: Vec::with_capacity(2 * n_to_alloc),
                i_shade,
                avg_min_grad_dir: 0.0,
            };
            // Adds first pixel
            shade_region.coords.push((x, y));
            alloc_pixels[x as usize][y as usize] = true;
            // Adds remaining pixels of the region
            shade_region.find_all_coords(&self.img, &i_shades, &mut alloc_pixels);
            // Adds region to regions vec
            self.shade_regions.push(shade_region);
        }

        Ok(())
    }

    /// Generates the vector of shade indexes based on an equaly spaced distribution of shade
    /// resolution reduction
    fn gen_i_shades(&self) -> Vec<Vec<u8>> {
        let step = std::u8::MAX / (self.n_shades - 1);
        let mut i_shades =
            vec![vec![0 as u8; self.img.height() as usize]; self.img.width() as usize];
        for x in 0..self.img.width() {
            for y in 0..self.img.height() {
                let mut i_shade = self.img.get_pixel(x, y)[0] / step;
                if i_shade == self.n_shades {
                    i_shade -= 1
                }
                i_shades[x as usize][y as usize] = i_shade;
            }
        }

        i_shades
    }

    /// Calculates the regions' average minimum shade gradient directions.
    fn calc_regions_avg_min_grad_dirs(&mut self) -> Result<(), Error> {
        // Generates the minimum shade gradient directions map
        println!("Calculating pixels' minimum shade gradient directions map...");
        let min_grad_map = self.gen_min_grad_map();
        // Finds average min grad direction for each region
        println!("Finding regions' average minimum shade gradient directions...");
        let pb = ProgressBar::new(self.shade_regions.len() as u64);
        for region in pb.wrap_iter(self.shade_regions.iter_mut()) {
            region.calc_avg_min_grad_dirs(&min_grad_map)?;
        }

        Ok(())
    }

    /// Generates the minimum shade gradient map for a grayscale image. This map contains the directions
    /// to where the shade changes less for each pixel, relative to the other analyzed directions.
    fn gen_min_grad_map(&self) -> Vec<Vec<f64>> {
        // Finds radius of gradient analysis
        let n_layers = self.n_grad_dir / 4 + 1;
        // Generates test directions vector
        let directs_to_eval = self.gen_directs_to_eval();
        // Finds the direction of minimum gradient for each pixel
        let mut min_grad_directs_map =
            vec![vec![0.0; self.img.height() as usize]; self.img.width() as usize];
        let pb = ProgressBar::new(self.img.pixels().len() as u64);
        for (x, y, _) in pb.wrap_iter(self.img.enumerate_pixels()) {
            // Gets current subset of pixels
            let mut pixel_subset = pixel_subset::PixelSubset::new(&self.img, n_layers);
            pixel_subset.fill((x, y)).unwrap();
            // Finds direction of minimal shade gradient
            let i_min_grad_dir = pixel_subset.find_i_min_grad_dir(&directs_to_eval);
            min_grad_directs_map[x as usize][y as usize] = directs_to_eval[i_min_grad_dir as usize];
        }

        min_grad_directs_map
    }

    /// Generates the directions for shade gradient evaluation.
    fn gen_directs_to_eval(&self) -> Vec<f64> {
        let step = PI / self.n_grad_dir as f64;
        let mut directs_to_eval = Vec::with_capacity(self.n_grad_dir as usize);
        for i_dir in 0..self.n_grad_dir {
            directs_to_eval.push(i_dir as f64 * step);
        }

        directs_to_eval
    }

    /// Makes the straight line image based on the grayscale image, the shades regions and the minimal
    /// gradient directions map.
    fn make_output_img(&mut self, stroke_width: u32) -> Result<(), Error> {
        println!("Making output image...");
        // Loops through the regions and finds the shade of each pixel
        let pb = ProgressBar::new(self.shade_regions.len() as u64);
        for region in pb.wrap_iter(self.shade_regions.iter()) {
            // Calculates width of the black substroke of the stroke
            let black_stroke_width =
                (region.i_shade as f64 / self.n_shades as f64 * stroke_width as f64) as u32;
            for (x, y) in region.coords.iter() {
                // Claculates index of the stroke shade
                let i_shade_stroke = ((*x as f64 * region.avg_min_grad_dir.cos()
                    + *y as f64 * region.avg_min_grad_dir.sin())
                    % stroke_width as f64) as u32;
                // Decides if pixel is black or white
                let shade = if i_shade_stroke <= black_stroke_width {
                    u8::MAX
                } else {
                    0
                };
                // Sets pixel color in image
                self.img.get_pixel_mut(*x, *y).0[0] = shade;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use crate::test_util;

    use super::*;

    #[test]
    fn gen_shade_regions_5x5_dir0() {
        let img_gs = test_util::tests::img_grad_factory(5, 5, 0.0);
        let n_shades = 5;

        let mut img_proc = ImageProcessor::build(DynamicImage::ImageLumaA8(img_gs), n_shades, 5);
        img_proc.gen_shade_regions().unwrap();

        let expected = vec![
            vec![(0, 0), (0, 1), (0, 2), (0, 3), (0, 4)],
            vec![(1, 0), (1, 1), (1, 2), (1, 3), (1, 4)],
            vec![(2, 0), (2, 1), (2, 2), (2, 3), (2, 4)],
            vec![(3, 0), (3, 1), (3, 2), (3, 3), (3, 4)],
            vec![(4, 0), (4, 1), (4, 2), (4, 3), (4, 4)],
        ];

        let result: Vec<Vec<(u32, u32)>> = img_proc
            .shade_regions
            .into_iter()
            .map(|region| region.coords)
            .collect();

        assert_eq!(expected, result);
    }

    #[test]
    fn gen_directs_to_eval4() {
        let img_gs = test_util::tests::img_grad_factory(5, 5, 0.0);
        let n_shades = 5;

        let img_proc = ImageProcessor::build(DynamicImage::ImageLumaA8(img_gs), n_shades, 4);

        let expected = vec![0.0, PI / 4.0, PI / 2.0, 3.0 * PI / 4.0];
        let result = img_proc.gen_directs_to_eval();

        assert_eq!(expected, result);
    }

    #[test]
    fn calc_regions_avg_min_grad_dirs_5x5_dir_pi4() {
        let img_gs = test_util::tests::img_grad_factory(5, 5, 0.0);
        let n_shades = 5;

        let mut img_proc = ImageProcessor::build(DynamicImage::ImageLumaA8(img_gs), n_shades, 2);
        img_proc.gen_shade_regions().unwrap();
        img_proc.calc_regions_avg_min_grad_dirs().unwrap();

        let expected = vec![0.0; 5];

        let result: Vec<f64> = img_proc
            .shade_regions
            .into_iter()
            .map(|region| region.avg_min_grad_dir)
            .collect();

        assert_eq!(expected, result);
    }
}
