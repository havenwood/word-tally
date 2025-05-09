//! Common utilities for benchmarking word-tally.

use std::io::{Cursor, Write};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use criterion::{BatchSize, Criterion, black_box, measurement::WallTime};
use fake::Fake;
use fake::faker::lorem::en::Words;
use tempfile::NamedTempFile;

use word_tally::{Options, WordTally};

/// Generate text with random words for benchmarking
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

/// Small text sample (approximately 300 lines, ~30KB)
pub fn small_text() -> String {
    generate_sample_text(300, 5..12)
}

/// Medium text sample (approximately 800 lines, ~100KB)
pub fn medium_text() -> String {
    generate_sample_text(800, 8..15)
}

/// Wrap a value in an Arc for shared ownership
pub fn make_shared<T>(value: T) -> Arc<T> {
    Arc::new(value)
}

/// Create a standard criterion configuration
pub fn standard_criterion_config() -> Criterion {
    Criterion::default()
        .configure_from_args()
        .sample_size(60)
        .measurement_time(Duration::from_secs(7))
        .warm_up_time(Duration::from_secs(3))
}

/// Helper function to run a WordTally benchmark with string input
pub fn bench_word_tally_with_string<F>(
    b: &mut criterion::Bencher<'_>,
    text: String,
    options: &Arc<Options>,
    setup_fn: F,
) where
    F: Fn(String) -> Cursor<String>,
{
    b.iter_batched(
        || setup_fn(text.clone()),
        |input| black_box(WordTally::new(input, options)),
        BatchSize::LargeInput,
    );
}

/// Create a standardized file for benchmarking with consistent word distribution
/// and a predictable size in kilobytes
pub fn create_benchmark_file(size_kb: usize) -> (NamedTempFile, PathBuf) {
    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");

    let text_size = size_kb * 1024;
    let approx_chars_per_line = 80;
    let approx_lines = text_size / approx_chars_per_line;
    let words_per_line = 8..14;

    let content = generate_sample_text(approx_lines, words_per_line);

    temp_file
        .write_all(content.as_bytes())
        .expect("Failed to write to temp file");
    temp_file.flush().expect("Failed to flush temp file");

    let path = temp_file.path().to_path_buf();
    (temp_file, path)
}

/// Helper function to create a benchmark group with a standard name format
pub fn create_bench_group<'a>(
    c: &'a mut Criterion,
    name: &str,
) -> criterion::BenchmarkGroup<'a, WallTime> {
    c.benchmark_group(name)
}
