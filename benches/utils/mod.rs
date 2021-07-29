use std::collections::HashMap;
use std::env;

use glassbench::*;
use image::io::Reader;
use image::{RgbImage, RgbaImage};

pub fn get_big_rgb_image() -> RgbImage {
    let cur_dir = env::current_dir().unwrap();
    let img = Reader::open(cur_dir.join("data/nasa-4928x3279.png"))
        .unwrap()
        .decode()
        .unwrap();
    img.to_rgb8()
}

pub fn get_big_rgba_image() -> RgbaImage {
    let cur_dir = env::current_dir().unwrap();
    let img = Reader::open(cur_dir.join("data/nasa-4928x3279.png"))
        .unwrap()
        .decode()
        .unwrap();
    img.to_rgba8()
}

pub fn get_small_rgba_image() -> RgbaImage {
    let cur_dir = env::current_dir().unwrap();
    let img = Reader::open(cur_dir.join("data/nasa-852x567.png"))
        .unwrap()
        .decode()
        .unwrap();
    img.to_rgba8()
}

pub fn print_md_table(bench: &Bench) {
    let mut res_map: HashMap<String, Vec<String>> = HashMap::new();
    let mut crate_names: Vec<String> = Vec::new();
    let mut alg_names: Vec<String> = Vec::new();

    for task in bench.tasks.iter() {
        if let Some(measure) = task.measure {
            let parts: Vec<&str> = task.name.split('-').map(|s| s.trim()).collect();
            let crate_name = parts[0].to_string();
            let alg_name = parts[1].to_string();
            let value = measure.total_duration.as_secs_f64() * 1000. / measure.iterations as f64;

            if !crate_names.contains(&crate_name) {
                crate_names.push(crate_name.clone());
            }
            if !alg_names.contains(&alg_name) {
                alg_names.push(alg_name);
            }

            if !res_map.contains_key(&crate_name) {
                res_map.insert(crate_name.clone(), Vec::new());
            }
            if let Some(values) = res_map.get_mut(&crate_name) {
                values.push(format!("{:.3}", value));
            }
        }
    }

    let first_column_width = res_map.keys().map(|s| s.len()).max().unwrap_or(0);
    let mut column_width: Vec<usize> = vec![first_column_width];

    for (i, name) in alg_names.iter().enumerate() {
        let width = res_map.values().map(|v| v[i].len()).max().unwrap_or(0);
        column_width.push(width.max(name.len()));
    }

    let mut first_row: Vec<String> = vec!["".to_owned()];
    alg_names.iter().for_each(|s| first_row.push(s.to_owned()));
    print_row(&column_width, &first_row);
    print_header_underline(&column_width);

    for name in crate_names.iter() {
        if let Some(values) = res_map.get(name) {
            let mut row = vec![name.clone()];
            values.iter().for_each(|s| row.push(s.clone()));
            print_row(&column_width, &row);
        }
    }
}

fn print_row(widths: &[usize], values: &[String]) {
    for (i, (&width, value)) in widths.iter().zip(values).enumerate() {
        if i == 0 {
            print!("| {:width$} ", value, width = width);
        } else {
            print!("| {:^width$} ", value, width = width);
        }
    }
    println!("|");
}

fn print_header_underline(widths: &[usize]) {
    for (i, &width) in widths.iter().enumerate() {
        if i == 0 {
            print!("|{:-<width$}", "", width = width + 2);
        } else {
            print!("|:{:-<width$}:", "", width = width);
        }
    }
    println!("|");
}
