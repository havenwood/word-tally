//! I/O strategy benchmarks.

use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use word_tally::{Input, Io, Options, WordTally};

#[path = "common.rs"]
pub mod common;
use self::common::{create_bench_group, make_shared, standard_criterion_config};

/// Benchmark I/O strategies for specified file size
fn bench_io_processing_combinations(c: &mut Criterion, size_kb: usize) {
    let (temp_file, file_path) = self::common::create_benchmark_file(size_kb);

    let file_content = std::fs::read(&file_path).expect("Failed to read benchmark file");

    let io_strategies = [
        (Io::Stream, "stream"),
        (Io::ParallelStream, "parallel-stream"),
        (Io::ParallelInMemory, "parallel-in-memory"),
        (Io::ParallelMmap, "parallel-mmap"),
        (Io::ParallelBytes, "parallel-bytes"),
    ];

    {
        let group_name = format!("io_strategies/file_size_{size_kb}kb");
        let mut group = create_bench_group(c, &group_name);

        for (io, io_name) in &io_strategies {
            let options = Options::default().with_io(*io);
            let shared_options = make_shared(options);

            if *io == Io::ParallelBytes {
                let file_content_clone = file_content.clone();
                group.bench_function(*io_name, |b| {
                    b.iter_batched(
                        || Input::from_bytes(&file_content_clone),
                        |input| {
                            black_box(
                                WordTally::new(&input, &shared_options)
                                    .expect("Failed to create WordTally"),
                            )
                        },
                        BatchSize::LargeInput,
                    );
                });
            } else {
                group.bench_function(*io_name, |b| {
                    b.iter_batched(
                        || Input::new(&file_path, *io).expect("Failed to create input"),
                        |input| {
                            black_box(
                                WordTally::new(&input, &shared_options)
                                    .expect("Failed to create WordTally"),
                            )
                        },
                        BatchSize::LargeInput,
                    );
                });
            }
        }

        group.finish();
    }

    drop(temp_file);
}

fn run_benchmarks(c: &mut Criterion) {
    bench_io_processing_combinations(c, 10);

    bench_io_processing_combinations(c, 75);

    #[cfg(not(debug_assertions))]
    {
        bench_io_processing_combinations(c, 500);
    }
}

criterion_group! {
    name = benches;
    config = standard_criterion_config();
    targets = run_benchmarks
}

criterion_main!(benches);
