//! I/O strategy benchmarks.

use criterion::{Criterion, criterion_group, criterion_main};
use word_tally::{Io, Options};

#[path = "common.rs"]
pub mod common;
use self::common::{
    bench_io_with_file, create_bench_group, make_shared, standard_criterion_config,
};

/// Benchmark I/O strategies for specified file size
fn bench_io_processing_combinations(c: &mut Criterion, size_kb: usize) {
    let (temp_file, file_path) = self::common::create_benchmark_file(size_kb);

    let group_name = format!("io_strategies/file_size_{size_kb}kb");
    {
        let mut group = create_bench_group(c, &group_name);

        let key_strategies = [
            (Io::Stream, "stream"),
            (Io::ParallelStream, "parallel-stream"),
            (Io::ParallelMmap, "parallel-mmap"),
        ];

        for (io, io_name) in &key_strategies {
            let options = Options::default().with_io(*io);
            let shared_options = make_shared(options);

            group.bench_function(*io_name, |b| {
                bench_io_with_file(b, &file_path, *io, &shared_options);
            });
        }

        group.finish();
    }
    drop(temp_file);
}

fn run_benchmarks(c: &mut Criterion) {
    bench_io_processing_combinations(c, 10);

    #[cfg(not(debug_assertions))]
    {
        bench_io_processing_combinations(c, 50);
    }
}

criterion_group! {
    name = benches;
    config = standard_criterion_config();
    targets = run_benchmarks
}

criterion_main!(benches);
