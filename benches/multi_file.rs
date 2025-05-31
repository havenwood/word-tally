//! Multi-file processing benchmarks.

use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use tempfile::NamedTempFile;
use word_tally::{Input, Options, TallyMap};

#[path = "common.rs"]
pub mod common;
use self::common::{
    IO_STRATEGIES_NO_BYTES, create_bench_group, make_shared, standard_criterion_config,
};

fn bench_multi_file_processing(c: &mut Criterion) {
    let file_sizes_kb = [5, 8, 7];
    let temp_files_and_paths: Vec<(NamedTempFile, std::path::PathBuf)> = file_sizes_kb
        .iter()
        .map(|&size| common::create_benchmark_file(size))
        .collect();

    let file_paths: Vec<&str> = temp_files_and_paths
        .iter()
        .map(|(_, path)| {
            path.to_str()
                .expect("benchmark file path should be valid UTF-8")
        })
        .collect();

    let mut group = create_bench_group(c, "multi_file_processing");

    for (io, name) in &IO_STRATEGIES_NO_BYTES {
        let options = Options::default().with_io(*io);
        let shared_options = make_shared(options);

        group.bench_function(*name, |b| {
            b.iter_batched(
                || {
                    file_paths
                        .iter()
                        .map(|path| Input::new(path, *io).expect("create input"))
                        .collect::<Vec<_>>()
                },
                |inputs| {
                    let tally_map = inputs
                        .iter()
                        .map(|input| TallyMap::from_input(input, &shared_options))
                        .try_fold(TallyMap::new(), |acc, result| {
                            result.map(|tally| acc.merge(tally))
                        })
                        .expect("process inputs");
                    black_box(tally_map)
                },
                BatchSize::LargeInput,
            );
        });
    }

    group.finish();
}

criterion_group! {
    name = benches;
    config = standard_criterion_config();
    targets = bench_multi_file_processing
}

criterion_main!(benches);
