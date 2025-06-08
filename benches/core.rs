//! Core benchmarks for sorting and filtering strategies.

use criterion::{Criterion, criterion_group, criterion_main};

use word_tally::Options;

#[path = "common.rs"]
mod common;
use self::common::{SORT_OPTIONS, bench_word_tally_with_string, small_text};

/// Benchmarks sorting strategies.
fn bench_sorting_strategies(c: &mut Criterion) {
    let mut group = c.benchmark_group("core/sorting_strategies");
    let text_sample = small_text();

    for (sort, sort_name) in &SORT_OPTIONS {
        let options = Options::default().with_sort(*sort);

        group.bench_function(*sort_name, |b| {
            bench_word_tally_with_string(b, &text_sample, &options);
        });
    }

    group.finish();
}

/// Benchmarks text processing strategies (filtering and case).
fn bench_text_processing_strategies(c: &mut Criterion) {
    let mut group = c.benchmark_group("core/text_processing");
    let text_sample = small_text();

    let strategies = [
        (Options::default(), "default"),
        (
            Options::default().with_case(word_tally::Case::Lower),
            "lowercase",
        ),
        (
            Options::default().with_filters(word_tally::Filters::default().with_min_chars(3)),
            "min_chars_filter",
        ),
        (
            Options::default()
                .with_case(word_tally::Case::Lower)
                .with_filters(
                    word_tally::Filters::default()
                        .with_min_chars(3)
                        .with_min_count(2),
                ),
            "combined_processing",
        ),
    ];

    for (options, strategy_name) in &strategies {
        group.bench_function(*strategy_name, |b| {
            bench_word_tally_with_string(b, &text_sample, options);
        });
    }

    group.finish();
}

fn run_benchmarks(c: &mut Criterion) {
    bench_sorting_strategies(c);
    bench_text_processing_strategies(c);
}

criterion_group! {
    name = benches;
    config = common::criterion_config();
    targets = run_benchmarks
}

criterion_main!(benches);
