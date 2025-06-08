//! Shared benchmark utilities for word-tally performance testing.

#![allow(dead_code)]

use std::fs;
use std::hint::black_box;
use std::io::Write;
use std::path::PathBuf;

use criterion::BatchSize;
use fake::Fake;
use fake::faker::lorem::en::Words;
use tempfile::NamedTempFile;

use word_tally::options::encoding::Encoding;
use word_tally::{Case, Filters, Io, Options, Reader, Sort, TallyMap, View, WordTally};

/// Standard Criterion configuration for all benchmarks.
#[must_use]
pub(crate) fn criterion_config() -> criterion::Criterion {
    criterion::Criterion::default()
        .configure_from_args()
        .sample_size(10)
        .measurement_time(std::time::Duration::from_secs(3))
        .warm_up_time(std::time::Duration::from_secs(1))
}

/// Generates random text for benchmarks.
#[must_use]
fn generate_sample_text(lines: usize, words_per_line: std::ops::Range<usize>) -> String {
    let mut result = String::with_capacity(lines * 80);

    for i in 0..lines {
        if i > 0 {
            result.push('\n');
        }
        let words: Vec<String> = Words(words_per_line.clone()).fake();
        for (j, word) in words.iter().enumerate() {
            if j > 0 {
                result.push(' ');
            }
            result.push_str(word);
        }
    }

    result
}

// Text size constants for clarity
const SMALL_TEXT_LINES: usize = 150;
const MEDIUM_TEXT_LINES: usize = 400;

/// Small text sample (~15KB).
#[must_use]
pub(crate) fn small_text() -> String {
    generate_sample_text(SMALL_TEXT_LINES, 5..12)
}

/// Medium text sample (~50KB).
#[must_use]
pub(crate) fn medium_text() -> String {
    generate_sample_text(MEDIUM_TEXT_LINES, 8..15)
}

/// Benchmarks `WordTally` with string input using the specified I/O mode.
pub(crate) fn bench_word_tally_with_string(
    b: &mut criterion::Bencher<'_>,
    text: &str,
    options: &Options,
) {
    let temp_file = NamedTempFile::new().expect("create benchmark temp file");
    std::fs::write(temp_file.path(), text.as_bytes()).expect("write benchmark text");
    let path = temp_file.path().to_path_buf();

    match options.io() {
        Io::Stream | Io::ParallelStream => bench_with_reader(
            b,
            || (path.as_path(), Some(text.as_bytes().to_vec())),
            options,
        ),
        Io::ParallelMmap => bench_with_mmap(
            b,
            || (path.as_path(), Some(text.as_bytes().to_vec())),
            options,
        ),
        Io::ParallelBytes => bench_with_bytes(
            b,
            || (path.as_path(), Some(text.as_bytes().to_vec())),
            options,
        ),
        Io::ParallelInMemory => bench_with_in_memory(
            b,
            || (path.as_path(), Some(text.as_bytes().to_vec())),
            options,
        ),
    }
}

/// Creates a benchmark file of specified size in KB.
#[must_use]
pub(crate) fn create_benchmark_file(size_kb: usize) -> (NamedTempFile, PathBuf) {
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

/// Processes a tally operation and returns the result for benchmarking.
#[inline]
#[must_use]
pub(crate) fn process_tally(tally_map: TallyMap, options: &Options) -> WordTally {
    black_box(WordTally::from_tally_map(tally_map, options))
}

/// Benchmarks using `Reader` for streaming modes.
fn bench_with_reader<F, P>(b: &mut criterion::Bencher<'_>, setup: F, options: &Options)
where
    F: Fn() -> (P, Option<Vec<u8>>),
    P: AsRef<std::path::Path>,
{
    b.iter_batched(
        || {
            let (path, _) = setup();
            Reader::try_from(path.as_ref()).expect("create reader")
        },
        |reader| {
            let tally_map = TallyMap::from_reader(&reader, options).expect("create tally map");
            process_tally(tally_map, options)
        },
        BatchSize::LargeInput,
    );
}

/// Benchmarks using memory-mapped `View`.
fn bench_with_mmap<F, P>(b: &mut criterion::Bencher<'_>, setup: F, options: &Options)
where
    F: Fn() -> (P, Option<Vec<u8>>),
    P: AsRef<std::path::Path>,
{
    b.iter_batched(
        || {
            let (path, _) = setup();
            View::try_from(path.as_ref()).expect("create view")
        },
        |view| {
            let tally_map = TallyMap::from_view(&view, options).expect("create tally map");
            process_tally(tally_map, options)
        },
        BatchSize::LargeInput,
    );
}

/// Benchmarks using byte slice `View`.
fn bench_with_bytes<F, P>(b: &mut criterion::Bencher<'_>, setup: F, options: &Options)
where
    F: Fn() -> (P, Option<Vec<u8>>),
    P: AsRef<std::path::Path>,
{
    let (_, bytes) = setup();
    let bytes = bytes.expect("ParallelBytes requires bytes");
    b.iter_batched(
        || View::from(&bytes[..]),
        |view| {
            let tally_map = TallyMap::from_view(&view, options).expect("create tally map");
            process_tally(tally_map, options)
        },
        BatchSize::LargeInput,
    );
}

/// Benchmarks using in-memory `View`.
fn bench_with_in_memory<F, P>(b: &mut criterion::Bencher<'_>, setup: F, options: &Options)
where
    F: Fn() -> (P, Option<Vec<u8>>),
    P: AsRef<std::path::Path>,
{
    let (path, bytes) = setup();
    if let Some(bytes) = bytes {
        let view = View::from(bytes);
        b.iter(|| {
            let tally_map = TallyMap::from_view(&view, options).expect("create tally map");
            process_tally(tally_map, options)
        });
    } else {
        b.iter_batched(
            || View::try_from(path.as_ref()).expect("create view"),
            |view| {
                let tally_map = TallyMap::from_view(&view, options).expect("create tally map");
                process_tally(tally_map, options)
            },
            BatchSize::LargeInput,
        );
    }
}

/// I/O strategies that work with file paths (excludes `ParallelBytes`).
pub(crate) const IO_STRATEGIES_FILE: [(Io, &str); 4] = [
    (Io::Stream, "stream"),
    (Io::ParallelStream, "parallel-stream"),
    (Io::ParallelInMemory, "parallel-in-memory"),
    (Io::ParallelMmap, "parallel-mmap"),
];

/// Common sort options for benchmarks.
pub(crate) const SORT_OPTIONS: [(Sort, &str); 3] = [
    (Sort::Unsorted, "unsorted"),
    (Sort::Desc, "descending"),
    (Sort::Asc, "ascending"),
];

/// Encoding strategies for benchmarks.
pub(crate) const ENCODING_OPTIONS: [(Encoding, &str); 2] =
    [(Encoding::Unicode, "unicode"), (Encoding::Ascii, "ascii")];

/// Processing mode options for benchmarks.
pub(crate) const PROCESSING_OPTIONS: [(Io, &str); 2] =
    [(Io::ParallelStream, "parallel"), (Io::Stream, "sequential")];

/// Case normalization options for benchmarks.
pub(crate) const CASE_OPTIONS: [(Case, &str); 3] = [
    (Case::Original, "original"),
    (Case::Lower, "lower"),
    (Case::Upper, "upper"),
];

/// Filter strategy options for benchmarks.
#[must_use]
pub(crate) fn filter_options() -> [(Filters, &'static str); 4] {
    [
        (Filters::default(), "none"),
        (Filters::default().with_min_chars(3), "min_chars"),
        (Filters::default().with_min_count(2), "min_count"),
        (
            Filters::default().with_min_count(2).with_min_chars(3),
            "combined",
        ),
    ]
}

/// Benchmarks I/O strategy with file.
pub(crate) fn bench_io_with_file(
    b: &mut criterion::Bencher<'_>,
    file_path: &std::path::Path,
    options: &Options,
) {
    let io = options.io();
    let file_content = if io == Io::ParallelBytes {
        Some(fs::read(file_path).expect("read benchmark file"))
    } else {
        None
    };

    match io {
        Io::Stream | Io::ParallelStream => {
            bench_with_reader(b, || (file_path, file_content.clone()), options);
        }
        Io::ParallelMmap => bench_with_mmap(b, || (file_path, file_content.clone()), options),
        Io::ParallelBytes => bench_with_bytes(b, || (file_path, file_content.clone()), options),
        Io::ParallelInMemory => {
            bench_with_in_memory(b, || (file_path, file_content.clone()), options);
        }
    }
}
