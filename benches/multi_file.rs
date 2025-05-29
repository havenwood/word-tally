//! Multi-file processing benchmarks.

use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use tempfile::NamedTempFile;
use word_tally::{Input, Io, Options, TallyMap};

#[path = "common.rs"]
pub mod common;
use self::common::{create_bench_group, make_shared, standard_criterion_config};

fn bench_multi_file_processing(c: &mut Criterion) {
    let file_sizes_kb = [10, 20, 15];
    let temp_files_and_paths: Vec<(NamedTempFile, std::path::PathBuf)> = file_sizes_kb
        .iter()
        .map(|&size| common::create_benchmark_file(size))
        .collect();

    let file_paths: Vec<&str> = temp_files_and_paths
        .iter()
        .map(|(_, path)| path.to_str().unwrap())
        .collect();

    let io_strategies = [
        (Io::Stream, "stream"),
        (Io::ParallelStream, "parallel-stream"),
        (Io::ParallelInMemory, "parallel-in-memory"),
        (Io::ParallelMmap, "parallel-mmap"),
    ];

    let mut group = create_bench_group(c, "multi_file_processing");

    for (io, name) in &io_strategies {
        let options = Options::default().with_io(*io);
        let shared_options = make_shared(options);

        group.bench_function(*name, |b| {
            b.iter_batched(
                || {
                    file_paths
                        .iter()
                        .map(|path| Input::new(path, *io).expect("Failed to create input"))
                        .collect::<Vec<_>>()
                },
                |inputs| {
                    let tally_map = inputs
                        .iter()
                        .map(|input| TallyMap::from_input(input, &shared_options))
                        .try_fold(TallyMap::new(), |acc, result| {
                            result.map(|tally| acc.merge(tally))
                        })
                        .expect("Failed to process inputs");
                    black_box(tally_map)
                },
                BatchSize::LargeInput,
            );
        });
    }

    group.finish();
}

fn bench_multi_file_scaling(c: &mut Criterion) {
    let file_counts = vec![2, 4, 8];
    let size_per_file_kb = 20;

    for count in file_counts {
        let temp_files_and_paths: Vec<(NamedTempFile, std::path::PathBuf)> = (0..count)
            .map(|_| common::create_benchmark_file(size_per_file_kb))
            .collect();

        let file_paths: Vec<&str> = temp_files_and_paths
            .iter()
            .map(|(_, path)| path.to_str().unwrap())
            .collect();

        let group_name = format!("multi_file_scaling/{count}_files");
        let mut group = create_bench_group(c, &group_name);

        let io_strategies = [
            (Io::Stream, "stream"),
            (Io::ParallelStream, "parallel-stream"),
            (Io::ParallelMmap, "parallel-mmap"),
        ];

        for (io, name) in &io_strategies {
            let options = Options::default().with_io(*io);
            let shared_options = make_shared(options);

            group.bench_function(*name, |b| {
                b.iter_batched(
                    || {
                        file_paths
                            .iter()
                            .map(|path| Input::new(path, *io).expect("Failed to create input"))
                            .collect::<Vec<_>>()
                    },
                    |inputs| {
                        let tally_map = inputs
                            .iter()
                            .map(|input| TallyMap::from_input(input, &shared_options))
                            .try_fold(TallyMap::new(), |acc, result| {
                                result.map(|tally| acc.merge(tally))
                            })
                            .expect("Failed to process inputs");
                        black_box(tally_map)
                    },
                    BatchSize::LargeInput,
                );
            });
        }

        group.finish();
    }
}

criterion_group! {
    name = benches;
    config = standard_criterion_config();
    targets = bench_multi_file_processing, bench_multi_file_scaling
}

criterion_main!(benches);
