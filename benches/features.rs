//! Feature benchmarks for processing modes.

use criterion::{Criterion, criterion_group, criterion_main};
use word_tally::Options;

#[path = "common.rs"]
mod common;
use self::common::{PROCESSING_OPTIONS, bench_word_tally_with_string, medium_text};

/// Benchmarks sequential vs parallel processing.
fn bench_processing_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("features/processing_comparison");
    let text_sample = medium_text();

    for (io_mode, mode_name) in &PROCESSING_OPTIONS {
        let options = Options::default().with_io(*io_mode);
        group.bench_function(*mode_name, |b| {
            bench_word_tally_with_string(b, &text_sample, &options);
        });
    }

    group.finish();
}

fn run_benchmarks(c: &mut Criterion) {
    bench_processing_comparison(c);
}

criterion_group! {
    name = benches;
    config = common::criterion_config();
    targets = run_benchmarks
}

criterion_main!(benches);
