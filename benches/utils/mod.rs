use std::env;

use image::io::Reader;
use image::RgbaImage;

pub fn get_big_rgb_image() -> RgbaImage {
    let cur_dir = env::current_dir().unwrap();
    let img = Reader::open(cur_dir.join("data/nasa-4928x3279.png"))
        .unwrap()
        .decode()
        .unwrap();
    img.to_rgba8()
}

pub fn get_small_rgb_image() -> RgbaImage {
    let cur_dir = env::current_dir().unwrap();
    let img = Reader::open(cur_dir.join("data/nasa-852x567.png"))
        .unwrap()
        .decode()
        .unwrap();
    img.to_rgba8()
}
