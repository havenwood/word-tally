//! Shared benchmark utilities.

use std::fs;
use std::hint::black_box;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use criterion::{BatchSize, Criterion, measurement::WallTime};
use fake::Fake;
use fake::faker::lorem::en::Words;
use tempfile::NamedTempFile;

use word_tally::{Io, Options, Reader, Sort, TallyMap, View, WordTally};

/// Generate random text for benchmarks.
#[must_use]
pub fn generate_sample_text(lines: usize, words_per_line: std::ops::Range<usize>) -> String {
    (0..lines)
        .map(|_| {
            Words(words_per_line.clone())
                .fake::<Vec<String>>()
                .join(" ")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Small text sample (~15KB)
#[must_use]
pub fn small_text() -> String {
    generate_sample_text(150, 5..12)
}

/// Medium text sample (~50KB)
#[must_use]
pub fn medium_text() -> String {
    generate_sample_text(400, 8..15)
}

/// Wrap value in Arc for sharing.
pub fn make_shared<T>(value: T) -> Arc<T> {
    Arc::new(value)
}

/// Standard Criterion configuration.
#[must_use]
pub fn standard_criterion_config() -> Criterion {
    Criterion::default()
        .configure_from_args()
        .sample_size(15)
        .measurement_time(Duration::from_secs(3))
        .warm_up_time(Duration::from_secs(1))
}

/// Benchmark `WordTally` with string input.
///
/// # Panics
///
/// Panics if tally map creation fails.
pub fn bench_word_tally_with_string<F>(
    b: &mut criterion::Bencher<'_>,
    text: &str,
    options: &Arc<Options>,
    setup_fn: F,
) where
    F: Fn(&str) -> (NamedTempFile, View),
{
    b.iter_batched(
        || setup_fn(text),
        |(temp_file, view)| {
            // Keep temp_file alive within the closure
            let _ = &temp_file;
            let tally_map = TallyMap::from_view(&view, options).expect("create tally map");
            black_box(WordTally::from_tally_map(tally_map, options))
        },
        BatchSize::LargeInput,
    );
}

/// Create benchmark file of specified size in KB.
///
/// # Panics
///
/// Panics if:
/// - Failed to create a temporary file
/// - Failed to write content to the file
/// - Failed to flush the file
#[must_use]
pub fn create_benchmark_file(size_kb: usize) -> (NamedTempFile, PathBuf) {
    let mut temp_file = NamedTempFile::new().expect("create temp file");

    let text_size = size_kb * 1024;
    let approx_chars_per_line = 80;
    let approx_lines = text_size / approx_chars_per_line;
    let words_per_line = 8..14;

    let content = generate_sample_text(approx_lines, words_per_line);

    temp_file
        .write_all(content.as_bytes())
        .expect("write to temp file");
    temp_file.flush().expect("flush temp file");

    let path = temp_file.path().to_path_buf();
    (temp_file, path)
}

/// Create named benchmark group.
pub fn create_bench_group<'a>(
    c: &'a mut Criterion,
    name: &str,
) -> criterion::BenchmarkGroup<'a, WallTime> {
    c.benchmark_group(name)
}

/// Create temp file and input from text.
///
/// # Panics
///
/// Panics if:
/// - Failed to create temporary file
/// - Failed to write to temporary file
/// - Failed to create View from file path
#[must_use]
pub fn create_temp_input(text: &str) -> (NamedTempFile, View) {
    let mut temp_file = NamedTempFile::new().expect("create benchmark temp file");
    Write::write_all(&mut temp_file, text.as_bytes()).expect("write benchmark text");
    let view = View::from(text.as_bytes());
    (temp_file, view)
}

/// Common I/O strategies for benchmarks.
pub const IO_STRATEGIES: [(Io, &str); 5] = [
    (Io::Stream, "stream"),
    (Io::ParallelStream, "parallel-stream"),
    (Io::ParallelInMemory, "parallel-in-memory"),
    (Io::ParallelMmap, "parallel-mmap"),
    (Io::ParallelBytes, "parallel-bytes"),
];

/// Common I/O strategies without bytes.
pub const IO_STRATEGIES_NO_BYTES: [(Io, &str); 4] = [
    (Io::Stream, "stream"),
    (Io::ParallelStream, "parallel-stream"),
    (Io::ParallelInMemory, "parallel-in-memory"),
    (Io::ParallelMmap, "parallel-mmap"),
];

/// Common sort options for benchmarks.
pub const SORT_OPTIONS: [(Sort, &str); 2] =
    [(Sort::Unsorted, "unsorted"), (Sort::Desc, "descending")];

/// Large text sample (~300KB)
#[must_use]
pub fn large_text() -> String {
    generate_sample_text(2400, 10..20)
}

/// Benchmark I/O strategy with file.
///
/// # Panics
///
/// Panics if:
/// - The benchmark file cannot be read
/// - Input creation fails
/// - Word tally creation fails
pub fn bench_io_with_file(
    b: &mut criterion::Bencher<'_>,
    file_path: &std::path::Path,
    io: Io,
    options: &Arc<Options>,
) {
    if io == Io::ParallelBytes {
        let file_content = fs::read(file_path).expect("read benchmark file");
        b.iter_batched(
            || View::from(&file_content[..]),
            |view| {
                let tally_map = TallyMap::from_view(&view, options).expect("create tally map");
                black_box(WordTally::from_tally_map(tally_map, options))
            },
            BatchSize::LargeInput,
        );
    } else if io == Io::ParallelMmap {
        b.iter_batched(
            || View::try_from(file_path).expect("create view"),
            |view| {
                let tally_map = TallyMap::from_view(&view, options).expect("create tally map");
                black_box(WordTally::from_tally_map(tally_map, options))
            },
            BatchSize::LargeInput,
        );
    } else {
        b.iter_batched(
            || Reader::try_from(file_path).expect("create reader"),
            |reader| {
                let tally_map = TallyMap::from_reader(&reader, options).expect("create tally map");
                black_box(WordTally::from_tally_map(tally_map, options))
            },
            BatchSize::LargeInput,
        );
    }
}
