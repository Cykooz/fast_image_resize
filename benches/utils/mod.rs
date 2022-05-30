use std::collections::HashMap;

use glassbench::*;

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
                let s_value = format!("{:.2}", value);
                if s_value == "0.00" {
                    values.push("-".to_string());
                } else {
                    values.push(s_value);
                }
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

/// Generates a benchmark with a consistent id
/// (using the benchmark file title), calling
/// the benchmarking functions given in argument.
///
/// ```no-test
/// bench_main!(
///     "Sortings",
///     bench_number_sorting,
///     bench_alpha_sorting,
/// );
/// ```
///
/// This generates the whole main function.
/// If you want to set the bench name yourself
/// (not recommanded), or change the way the launch
/// arguments are used, you can write the main
/// yourself and call [create_bench] and [after_bench]
/// instead of using this macro.
#[macro_export]
macro_rules! bench_main {
    (
        $title: literal,
        $( $fun: path, )+
    ) => {
        pub fn main() {
            // Pin process to #0 CPU core
            let mut cpu_set = nix::sched::CpuSet::new();
            cpu_set.set(0).unwrap();
            nix::sched::sched_setaffinity(nix::unistd::Pid::from_raw(0), &cpu_set).unwrap();
            glassbench!($title, $($fun,)+);
            main();
        }
    }
}
