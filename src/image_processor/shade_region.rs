use image::GrayAlphaImage;

/// Represents a shade region of a grayscale image.
#[derive(PartialEq, Debug)]
pub struct ShadeRegion {
    // Coordinates of the pixels contained in the region
    pub coords: Vec<(u32, u32)>,
    // Shade index (0 to n_shades-1)
    pub i_shade: u8,
    // Average direction of the pixels' minimum color gradient lines
    pub avg_min_grad_dir: f64,
}

impl ShadeRegion {
    // Finds the pixels that belong to this region
    pub fn find_all_coords(
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

#[cfg(test)]
mod tests {
    use image::DynamicImage;

    use crate::{image_processor::ImageProcessor, test_util};

    use super::*;

    #[test]
    fn shade_region_find_neighbors() {
        let img_gs = test_util::tests::img_grad_factory(3, 3, 0.0);

        let shade_region = ShadeRegion {
            coords: vec![(0, 0)],
            i_shade: 0,
            avg_min_grad_dir: 0.0,
        };

        let img_proc = ImageProcessor::build(DynamicImage::ImageLumaA8(img_gs), 3, 3);
        let i_shades = img_proc.gen_i_shades();
        let mut alloc_pixels =
            vec![vec![false; img_proc.img.height() as usize]; img_proc.img.width() as usize];

        for shade_vec in &i_shades {
            println!("i_shades_col: {:?}", shade_vec);
        }

        let expected = vec![(0, 0), (0, 2)];
        let result =
            shade_region.find_neighbors(&(0, 1), &img_proc.img, &i_shades, &mut alloc_pixels);

        assert_eq!(expected, result);
    }

    #[test]
    fn shade_region_find_all_coords() {
        let img_gs = test_util::tests::img_grad_factory(5, 5, 0.0);

        let img_proc = ImageProcessor::build(DynamicImage::ImageLumaA8(img_gs), 5, 3);
        let i_shades = img_proc.gen_i_shades();
        let mut alloc_pixels =
            vec![vec![false; img_proc.img.height() as usize]; img_proc.img.width() as usize];

        let mut shade_region = ShadeRegion {
            coords: vec![(0, 0)],
            i_shade: 0,
            avg_min_grad_dir: 0.0,
        };

        alloc_pixels[0][0] = true;

        for shade_vec in &i_shades {
            println!("i_shades_col: {:?}", shade_vec);
        }
        shade_region.find_all_coords(&img_proc.img, &i_shades, &mut alloc_pixels);

        let expected = vec![(0, 0), (0, 1), (0, 2), (0, 3), (0, 4)];
        let result = shade_region.coords;

        assert_eq!(expected, result);
    }
}
