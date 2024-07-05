use std::env;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use criterion::measurement::WallTime;
use criterion::{Bencher, BenchmarkGroup, BenchmarkId, Criterion};

use super::{cargo_target_directory, get_arch_id_and_name, get_results, BenchResult};

pub struct BenchGroup<'a> {
    pub criterion_group: BenchmarkGroup<'a, WallTime>,
    old_results: Vec<BenchResult>,
    results: Vec<BenchResult>,
}

impl<'a> BenchGroup<'a> {
    fn finish(self) -> Vec<BenchResult> {
        self.criterion_group.finish();
        self.results
    }
}

pub fn run_bench<F>(bench_fn: F, name: &str) -> Vec<BenchResult>
where
    F: FnOnce(&mut BenchGroup),
{
    if env::var("PIN_TO_CPU0").is_ok() {
        pin_process_to_cpu0();
    }

    let arch_id = get_arch_id_and_name().0;
    let output_dir = criterion_output_directory().join(arch_id);
    let mut criterion = Criterion::default()
        .output_directory(&output_dir)
        .configure_from_args();

    let now = SystemTime::now();
    let results_dir = output_dir.join(name);

    let results_lifetime: u32 = env::var("RESULTS_LIFETIME")
        .unwrap_or_else(|_| "0".to_owned())
        .parse()
        .unwrap_or_default();
    let old_results = if results_lifetime > 0 && name.starts_with("Compare ") {
        let old_now = now - Duration::from_secs(results_lifetime as u64 * 24 * 3600);
        get_results(&results_dir, &old_now)
    } else {
        vec![]
    };

    let mut group = BenchGroup {
        criterion_group: criterion.benchmark_group(name),
        old_results,
        results: vec![],
    };
    bench_fn(&mut group);
    let mut results = group.finish();
    criterion.final_summary();

    let new_results = get_results(&results_dir, &now);
    if new_results.is_empty() {
        new_results
    } else {
        for res in results.iter_mut().filter(|r| r.estimate < 0.) {
            res.estimate = new_results
                .iter()
                .find(|new_res| {
                    new_res.function_name == res.function_name && new_res.parameter == res.parameter
                })
                .map(|r| r.estimate)
                .unwrap_or(0.)
        }
        results
    }
}

pub fn bench<S1, S2, F>(
    group: &mut BenchGroup,
    sample_size: usize,
    func_name: S1,
    parameter: S2,
    mut f: F,
) where
    S1: Into<String>,
    S2: Into<String>,
    F: FnMut(&mut Bencher),
{
    let parameter = parameter.into();
    let func_name = func_name.into();
    // Use old results only for other libraries, not for 'fast_image_resize'
    if !func_name.starts_with("fir ") {
        if let Some(old_res) = group
            .old_results
            .iter()
            .find(|res| res.function_name == func_name && res.parameter == parameter)
        {
            group.results.push(old_res.clone());
            println!(
                "SKIP benching of '{}' function with '{}' parameter due to using old result.",
                func_name, parameter
            );
            return;
        }
    }

    group.results.push(BenchResult {
        function_name: func_name.clone(),
        parameter: parameter.clone(),
        estimate: -1., // Unknown result
    });

    group.criterion_group.sample_size(sample_size);
    group.criterion_group.bench_with_input(
        BenchmarkId::new(func_name, &parameter),
        &parameter,
        |bencher, _| f(bencher),
    );
}

/// Pin process to #0 CPU core
pub fn pin_process_to_cpu0() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut cpu_set = nix::sched::CpuSet::new();
        cpu_set.set(0).unwrap();
        nix::sched::sched_setaffinity(nix::unistd::Pid::from_raw(0), &cpu_set).unwrap();
    }
}

fn criterion_output_directory() -> PathBuf {
    if let Some(value) = env::var_os("CRITERION_HOME") {
        PathBuf::from(value)
    } else if let Some(path) = cargo_target_directory() {
        path.join("criterion")
    } else {
        PathBuf::from("target/criterion")
    }
}
