use std::borrow::Cow;
use std::collections::HashMap;

use itertools::Itertools;

#[derive(Debug, Clone)]
pub struct BenchResult {
    pub function_name: String,
    pub parameter: String,
    /// Estimate time in nanoseconds
    pub estimate: f64,
}

static COL_ORDER: [&str; 5] = ["Nearest", "Box", "Bilinear", "Bicubic", "Lanczos3"];

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
            if value >= 0.01 {
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
        match i {
            0 => buffer.push(format!("| {:width$} ", value, width = width)),
            _ => buffer.push(format!("| {:^width$} ", value, width = width)),
        }
    }
    buffer.push("|\n".to_string());
}

fn table_header_underline(buffer: &mut Vec<String>, widths: &[usize]) {
    for (i, &width) in widths.iter().enumerate() {
        match i {
            0 => buffer.push(format!("|{:-<width$}", "", width = width + 2)),
            _ => buffer.push(format!("|:{:-<width$}:", "", width = width)),
        }
    }
    buffer.push("|\n".to_string());
}
