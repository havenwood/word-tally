//! Core benchmarks for sorting and filtering strategies.

use criterion::{Criterion, criterion_group, criterion_main};
use word_tally::{Filters, Io, Options};

#[path = "common.rs"]
pub mod common;
use self::common::{
    SORT_OPTIONS, create_bench_group, create_temp_input, make_shared, small_text,
    standard_criterion_config,
};

/// Benchmark sorting strategies
fn bench_sorting_strategies(c: &mut Criterion) {
    let mut group = create_bench_group(c, "core/sorting_strategies");
    let text_sample = small_text();

    for (sort, sort_name) in &SORT_OPTIONS {
        let options = Options::default().with_sort(*sort).with_io(Io::Stream);

        let shared_options = make_shared(options);

        group.bench_function(*sort_name, |b| {
            common::bench_word_tally_with_string(
                b,
                &text_sample,
                &shared_options,
                create_temp_input,
            );
        });
    }

    group.finish();
}

/// Benchmark filtering strategies
fn bench_filtering_strategies(c: &mut Criterion) {
    let mut group = create_bench_group(c, "core/filtering_strategies");
    let text_sample = small_text();

    let filters = [
        (Filters::default(), "none"),
        (
            Filters::default().with_min_count(2).with_min_chars(3),
            "combined",
        ),
    ];

    for (filter, filter_name) in &filters {
        let options = Options::default()
            .with_filters(filter.clone())
            .with_io(Io::Stream);

        let shared_options = make_shared(options);

        group.bench_function(*filter_name, |b| {
            common::bench_word_tally_with_string(
                b,
                &text_sample,
                &shared_options,
                create_temp_input,
            );
        });
    }

    group.finish();
}

fn run_benchmarks(c: &mut Criterion) {
    bench_sorting_strategies(c);
    bench_filtering_strategies(c);
}

criterion_group! {
    name = benches;
    config = standard_criterion_config();
    targets = run_benchmarks
}

criterion_main!(benches);
