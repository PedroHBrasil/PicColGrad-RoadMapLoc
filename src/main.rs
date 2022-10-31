mod postprocessing;
mod preprocessing;
mod processing;

fn main() {
    // Pre-Processing
    let input = preprocessing::load_input();
    let img = preprocessing::load_image(&input.img_path);

    // Processing
    let img_gs = processing::process_img(img);

    // Post-Processing
    postprocessing::save_img(img_gs);
}
