use std::env;
use std::path::PathBuf;
use std::time::SystemTime;

use criterion::measurement::WallTime;
use criterion::{Bencher, BenchmarkGroup, BenchmarkId, Criterion};

use super::{cargo_target_directory, get_arch_name, get_results, BenchResult};

pub type BenchGroup<'a> = BenchmarkGroup<'a, WallTime>;

pub fn run_bench<F>(bench_fn: F, name: &str) -> Vec<BenchResult>
where
    F: FnOnce(&mut BenchGroup),
{
    pin_process_to_cpu0();

    let arch_name = get_arch_name();
    let output_dir = criterion_output_directory().join(arch_name);
    let mut criterion = Criterion::default()
        .output_directory(&output_dir)
        .configure_from_args();

    let now = SystemTime::now();

    let mut group = criterion.benchmark_group(name);
    bench_fn(&mut group);
    group.finish();
    criterion.final_summary();

    let results_dir = output_dir.join(name);
    get_results(&results_dir, &now)
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
    group.sample_size(sample_size);
    group.bench_with_input(
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
