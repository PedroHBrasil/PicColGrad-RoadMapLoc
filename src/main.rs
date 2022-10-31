mod postprocessing;
mod preprocessing;
mod processing;

fn main() {
    // Pre-Processing
    let input = preprocessing::load_input();
    let mut img = preprocessing::load_image(&input.img_path);

    // Processing
    img = processing::process_img(img);

    // Post-Processing
    postprocessing::save_img(img);
}
