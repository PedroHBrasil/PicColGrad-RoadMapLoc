mod postprocessing;
mod preprocessing;
mod processing;

fn main() {
    let input = preprocessing::load_input();
    let img = preprocessing::load_image(&input.img_path);

    let img_gs = processing::process_img(img);

    postprocessing::save_img(img_gs);
}
