//! Multi-file processing benchmarks.

use core::convert::AsRef;
use std::path::PathBuf;

use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use tempfile::NamedTempFile;
use word_tally::{Buffered, Io, Mapped, Options, TallyMap};

#[path = "common.rs"]
mod common;
use self::common::{IO_STRATEGIES_FILE, create_benchmark_file, process_tally};

/// Processes multiple sources following the main.rs pattern
fn process_multi_file_sources(sources: &[&str], options: &Options) -> anyhow::Result<TallyMap> {
    if options.io() == Io::ParallelMmap {
        let views: Result<Vec<_>, _> = sources.iter().map(|path| Mapped::try_from(*path)).collect();
        let views = views?;

        views
            .iter()
            .map(|view| TallyMap::from_mapped_input(view, options))
            .try_fold(TallyMap::new(), |acc, result| {
                result.map(|tally| acc.merge(tally))
            })
    } else {
        let readers: Result<Vec<_>, _> = sources
            .iter()
            .map(|path| Buffered::try_from(*path))
            .collect();
        let readers = readers?;

        readers
            .iter()
            .map(|reader| TallyMap::from_buffered_input(reader, options))
            .try_fold(TallyMap::new(), |acc, result| {
                result.map(|tally| acc.merge(tally))
            })
    }
}

fn bench_multi_file_processing(c: &mut Criterion) {
    let file_sizes_kb = [5, 8, 7];
    let temp_files_and_paths: Vec<(NamedTempFile, PathBuf)> = file_sizes_kb
        .iter()
        .map(|&size| create_benchmark_file(size))
        .collect();

    let file_paths: Vec<&str> = temp_files_and_paths
        .iter()
        .map(|(_, path)| {
            path.to_str()
                .expect("benchmark file path should be valid UTF-8")
        })
        .collect();

    let mut group = c.benchmark_group("multi_file/processing");

    for (io, name) in &IO_STRATEGIES_FILE {
        let options = Options::default().with_io(*io);

        group.bench_function(*name, |b| {
            b.iter_batched(
                || file_paths.clone(),
                |paths| {
                    let sources: Vec<&str> = paths.iter().map(AsRef::as_ref).collect();
                    let tally_map = process_multi_file_sources(&sources, &options)
                        .expect("process multi-file sources");
                    process_tally(tally_map, &options)
                },
                BatchSize::LargeInput,
            );
        });
    }

    group.finish();
}

criterion_group! {
    name = benches;
    config = common::criterion_config();
    targets = bench_multi_file_processing
}

criterion_main!(benches);
