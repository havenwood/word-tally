//! Benchmarks comparing I/O strategies (Streamed, Buffered, Memory-mapped)
//! and processing modes (Sequential, Parallel) across different file sizes.

use criterion::{BatchSize, Criterion, black_box, criterion_group, criterion_main};
use word_tally::{Input, Io, Options, Processing, WordTally};

#[path = "common.rs"]
pub mod common;
use self::common::{create_bench_group, make_shared, standard_criterion_config};

/// Run benchmarks for different I/O and processing combinations with a file of specified size
fn bench_io_processing_combinations(c: &mut Criterion, size_kb: usize) {
    // Create a test file with consistent content
    let (temp_file, file_path) = self::common::create_benchmark_file(size_kb);

    // Read the file content for bytes benchmark
    let file_content = std::fs::read(&file_path).expect("Failed to read benchmark file");

    // Define the benchmark test combinations
    let io_strategies = [
        (Io::Streamed, "streamed"),
        (Io::Buffered, "buffered"),
        (Io::MemoryMapped, "mmap"),
        (Io::Bytes, "bytes"),
    ];

    let processing_strategies = [
        (Processing::Sequential, "sequential"),
        (Processing::Parallel, "parallel"),
    ];

    // Create and use group in the same scope to address drop tightening warning
    {
        let group_name = format!("size_{}kb", size_kb);
        let mut group = create_bench_group(c, &group_name);

        // Test all combinations of I/O and processing strategies
        for (io, io_name) in &io_strategies {
            for (processing, proc_name) in &processing_strategies {
                let benchmark_name = format!("{}_{}", io_name, proc_name);
                let options = Options::default().with_io(*io).with_processing(*processing);
                let shared_options = make_shared(options);

                if *io == Io::Bytes {
                    // Use from_bytes for Bytes I/O mode
                    let file_content_clone = file_content.clone();
                    group.bench_function(&benchmark_name, |b| {
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
                    // Use Input::new for file-based I/O modes
                    group.bench_function(&benchmark_name, |b| {
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
        }

        group.finish();
    }

    // Ensure temporary file stays alive until the end of the benchmark
    drop(temp_file);
}

fn run_benchmarks(c: &mut Criterion) {
    // Small file benchmark (10KB)
    bench_io_processing_combinations(c, 10);

    // Medium file benchmark (75KB)
    bench_io_processing_combinations(c, 75);

    // Only run large file benchmark in release mode
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
