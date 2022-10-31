mod postprocessing;
mod preprocessing;
mod processing;

fn main() {
    let input = preprocessing::load_input();
    let mut img = preprocessing::load_image(&input.img_path);

    img = processing::process_img(img);

    postprocessing::save_img(img);
}
