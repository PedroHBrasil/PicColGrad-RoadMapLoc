mod image_processor;
mod input_reader;
mod output_writer;
mod test_util;

fn main() {
    // Pre-Processing
    let input = input_reader::load_input();
    let img = input_reader::load_image(&input.img_path);

    // Processing
    let img_gs =
        image_processor::run(img, input.n_shades, input.n_grad_dir, input.stroke_width).unwrap();

    // Post-Processing
    output_writer::save_img(img_gs, &input.out_file_name);
}
