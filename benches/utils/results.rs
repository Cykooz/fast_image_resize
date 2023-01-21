use std::borrow::Cow;
use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use itertools::Itertools;
use serde::Deserialize;
use walkdir::WalkDir;

use super::get_arch_name;

#[derive(Debug)]
pub struct BenchResult {
    pub function_name: String,
    pub parameter: String,
    /// Estimate time in nanoseconds
    pub estimate: f64,
}

impl BenchResult {
    pub fn new(function_name: String, parameter: Option<String>, path: &Path) -> Self {
        #[derive(Deserialize)]
        struct Mean {
            point_estimate: f64,
        }

        #[derive(Deserialize)]
        struct Estimates {
            mean: Mean,
        }

        let data =
            std::fs::read_to_string(path).expect("Unable to read file with benchmark results");

        let estimates: Estimates =
            serde_json::from_str(&data).expect("Unable to parse JSON data with benchmark results");

        Self {
            function_name,
            parameter: parameter.unwrap_or_default(),
            estimate: estimates.mean.point_estimate,
        }
    }
}

/// Find all "new/estimates.json" files inside of given directory.
/// Get only files what were created after the given time.
/// Read estimate time from this files and return vector of `BenchResult` instances.
pub fn get_results(parent_dir: &PathBuf, modified_after: &SystemTime) -> Vec<BenchResult> {
    let mut result = vec![];
    if !parent_dir.is_dir() {
        println!("WARNING: Directory with bench results is absent");
        return result;
    }

    let result_paths = WalkDir::new(parent_dir)
        .follow_links(true)
        .into_iter()
        .map(|e| e.expect("Invalid FS entry"))
        .filter(|e| e.path().ends_with("new/estimates.json"))
        .map(|e| (e.metadata().expect("Unable get metadata for FS entity"), e))
        .filter(|(m, _)| m.is_file())
        .map(|(m, e)| {
            (
                m.modified()
                    .expect("Unable to get last modification time of estimates.json file"),
                e,
            )
        })
        // Exclude old results
        .filter(|(modified, _)| modified >= modified_after)
        .sorted_by_key(|(modified, _)| modified.to_owned())
        .map(|(_, e)| e.into_path());

    for path in result_paths {
        let rel_path = path.strip_prefix(parent_dir).unwrap_or(&path).to_path_buf();
        let path_components: Vec<String> = rel_path
            .iter()
            .map(|os_str| {
                os_str
                    .to_str()
                    .expect("Unable to convert FS entry name into String")
                    .to_string()
            })
            .collect();
        let (function_name, parameter_name) = match path_components.as_slice() {
            [f, p, _, _] => (f.to_string(), Some(p.to_string())),
            [f, _, _] => (f.to_string(), None),
            _ => panic!("Relative path to bench result is invalid"),
        };

        result.push(BenchResult::new(function_name, parameter_name, &path));
    }

    result
}

static COL_ORDER: [&str; 4] = ["Nearest", "Bilinear", "CatmullRom", "Lanczos3"];

pub fn build_md_table(bench_results: &[BenchResult]) -> String {
    let mut row_names: Vec<String> = Vec::new();
    let mut row_indexes: HashMap<String, usize> = HashMap::new();
    let mut col_names: Vec<String> = Vec::new();

    for result in bench_results {
        let row_name = result.function_name.clone();
        if !row_names.contains(&row_name) {
            row_names.push(row_name.clone());
            row_indexes.insert(row_name.clone(), row_names.len() - 1);
        }
        let col_name = result.parameter.clone();
        if !col_names.contains(&col_name) {
            col_names.push(col_name.clone());
        }
    }

    // Reorder columns
    let mut ordered_pos = 0;
    for name in COL_ORDER {
        if let Some((cur_pos, _)) = col_names.iter().find_position(|s| s.as_str() == name) {
            if cur_pos != ordered_pos {
                col_names.swap(cur_pos, ordered_pos);
            }
            ordered_pos += 1;
        }
    }
    let col_indexes: HashMap<String, usize> = col_names
        .iter()
        .enumerate()
        .map(|(i, v)| (v.clone(), i))
        .collect();

    let cols_count = col_names.len();
    let mut values = vec![Cow::Borrowed("-"); row_names.len() * cols_count];

    for result in bench_results {
        let row_index = row_indexes.get(&result.function_name).copied();
        let col_index = col_indexes.get(&result.parameter).copied();
        if let (Some(row_index), Some(col_index)) = (row_index, col_index) {
            let value = result.estimate / 1000000.;
            if value >= 0.10 {
                let value_index = row_index * cols_count + col_index;
                values[value_index] = Cow::Owned(format!("{:.2}", value));
            }
        }
    }

    let first_column_width = row_names.iter().map(|s| s.len()).max().unwrap_or(0);
    let mut column_width: Vec<usize> = vec![first_column_width];

    for (col_index, col_name) in col_names.iter().enumerate() {
        let width = (0..row_names.len())
            .map(|row_index| {
                let value_index = row_index * cols_count + col_index;
                values.get(value_index).map(|v| v.len()).unwrap_or(0)
            })
            .max()
            .unwrap_or(0);
        column_width.push(width.max(col_name.len()));
    }

    let mut first_row: Vec<String> = vec!["".to_owned()];
    col_names.iter().for_each(|s| first_row.push(s.to_owned()));

    let mut str_buffer: Vec<String> = vec![];
    table_row(&mut str_buffer, &column_width, &first_row);
    table_header_underline(&mut str_buffer, &column_width);

    for row_name in row_names.iter() {
        let mut row = vec![row_name.clone()];
        for col_name in col_names.iter() {
            let row_index = row_indexes.get(row_name).copied();
            let col_index = col_indexes.get(col_name).copied();
            if let (Some(row_index), Some(col_index)) = (row_index, col_index) {
                let value_index = row_index * cols_count + col_index;
                let value = values
                    .get(value_index)
                    .map(|v| v.to_string())
                    .unwrap_or_default();
                row.push(value);
            }
        }
        table_row(&mut str_buffer, &column_width, &row);
    }

    str_buffer.join("")
}

fn table_row(buffer: &mut Vec<String>, widths: &[usize], values: &[String]) {
    for (i, (&width, value)) in widths.iter().zip(values).enumerate() {
        if i == 0 {
            buffer.push(format!("| {:width$} ", value, width = width));
        } else {
            buffer.push(format!("| {:^width$} ", value, width = width));
        }
    }
    buffer.push("|\n".to_string());
}

fn table_header_underline(buffer: &mut Vec<String>, widths: &[usize]) {
    for (i, &width) in widths.iter().enumerate() {
        if i == 0 {
            buffer.push(format!("|{:-<width$}", "", width = width + 2));
        } else {
            buffer.push(format!("|:{:-<width$}:", "", width = width));
        }
    }
    buffer.push("|\n".to_string());
}

fn insert_string_into_file(path: &Path, placeholder_name: &str, string: &str) {
    let mut content = std::fs::read_to_string(path).expect("Unable to read file into string");
    let start_maker = format!("<!-- {} start -->\n", placeholder_name);
    let start = match content.find(&start_maker) {
        Some(s) => s,
        None => {
            println!(
                "WARNING: Can't find start marker for placeholder '{}' in file {:?}",
                placeholder_name, path
            );
            return;
        }
    };
    let end_maker = format!("<!-- {} end -->", placeholder_name);
    let end = match content.find(&end_maker) {
        Some(s) => s,
        None => {
            println!(
                "WARNING: Can't find end marker for placeholder '{}' in file {:?}",
                placeholder_name, path
            );
            return;
        }
    };
    let replace_str = [start_maker.as_str(), string].join("");
    content.replace_range(start..end, &replace_str);
    std::fs::write(path, content).expect("Unable to save string into file");
}

fn write_bench_results_into_file(md_table: &str) {
    let file_name = format!("benchmarks-{}.md", get_arch_name());
    let file_path = PathBuf::from(file_name);
    if !file_path.is_file() {
        panic!("Can't find file {:?} in current directory", file_path);
    }
    let crate_name = env!("CARGO_CRATE_NAME");
    insert_string_into_file(file_path.as_path(), crate_name, md_table);

    if get_arch_name() == "x86_64" {
        let file_path = PathBuf::from("README.md");
        if !file_path.is_file() {
            panic!("Can't find file {:?} in current directory", file_path);
        }
        insert_string_into_file(file_path.as_path(), crate_name, md_table);
    }
}

pub fn print_and_write_compare_result(bench_results: &[BenchResult]) {
    if !bench_results.is_empty() {
        let md_table = build_md_table(bench_results);
        println!("{}", md_table);
        if env::var("WRITE_COMPARE_RESULT").unwrap_or_else(|_| "".to_owned()) == "1" {
            write_bench_results_into_file(&md_table);
        }
    }
}
