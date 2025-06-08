//! I/O strategy benchmarks.

use criterion::{Criterion, criterion_group, criterion_main};
use word_tally::Options;

#[path = "common.rs"]
mod common;
use self::common::{IO_STRATEGIES_FILE, bench_io_with_file, create_benchmark_file};

/// Benchmark I/O strategies for specified file size
fn bench_io_processing_combinations(c: &mut Criterion, size_kb: usize) {
    let (_temp_file, file_path) = create_benchmark_file(size_kb);

    let group_name = format!("io_strategies/file_size_{size_kb}kb");
    let mut group = c.benchmark_group(&group_name);

    for (io, io_name) in &IO_STRATEGIES_FILE {
        let options = Options::default().with_io(*io);

        group.bench_function(*io_name, |b| {
            bench_io_with_file(b, &file_path, &options);
        });
    }

    group.finish();
}

fn run_benchmarks(c: &mut Criterion) {
    bench_io_processing_combinations(c, 10);
}

criterion_group! {
    name = benches;
    config = common::criterion_config();
    targets = run_benchmarks
}

criterion_main!(benches);
