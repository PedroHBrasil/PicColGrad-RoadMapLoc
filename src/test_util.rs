#[cfg(test)]
pub mod tests {
    use image::{GrayAlphaImage, ImageBuffer, LumaA};

    pub fn img_grad_factory(width: u32, height: u32, min_grad_dir: f64) -> GrayAlphaImage {
        // Finds ratios and virtual image sizes
        let x_ratio =
            min_grad_dir.cos().abs() / (min_grad_dir.cos().abs() + min_grad_dir.sin().abs());
        let y_ratio =
            min_grad_dir.sin().abs() / (min_grad_dir.cos().abs() + min_grad_dir.sin().abs());
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
}
