//! Core benchmarks for sorting and filtering strategies.

use criterion::{Criterion, criterion_group, criterion_main};
use word_tally::{Filters, Input, Io, Options, Processing, Sort};

#[path = "common.rs"]
pub mod common;
use self::common::{create_bench_group, make_shared, small_text, standard_criterion_config};

/// Benchmark different sorting strategies
fn bench_sorting_strategies(c: &mut Criterion) {
    let mut group = create_bench_group(c, "sorting");
    let text_sample = small_text();

    let sort_options = [(Sort::Unsorted, "unsorted"), (Sort::Desc, "descending")];

    for (sort, sort_name) in &sort_options {
        let options = Options::default()
            .with_sort(*sort)
            .with_processing(Processing::Sequential);

        let shared_options = make_shared(options);

        group.bench_function(format!("sequential_{}", sort_name), |b| {
            self::common::bench_word_tally_with_string(
                b,
                text_sample.clone(),
                &shared_options,
                |text| {
                    let mut temp_file = tempfile::NamedTempFile::new().unwrap();
                    std::io::Write::write_all(&mut temp_file, text.as_bytes()).unwrap();
                    let input = Input::new(temp_file.path(), Io::Buffered).unwrap();
                    (temp_file, input)
                },
            );
        });
    }

    group.finish();
}

/// Benchmark different filtering strategies
fn bench_filtering_strategies(c: &mut Criterion) {
    let mut group = create_bench_group(c, "filtering");
    let text_sample = small_text();

    let min_count_filter = Filters::default().with_min_count(2);
    let min_chars_filter = Filters::default().with_min_chars(3);
    let combined_filter = Filters::default().with_min_count(2).with_min_chars(3);

    let filters = [
        (Filters::default(), "none"),
        (min_count_filter, "min_count"),
        (min_chars_filter, "min_chars"),
        (combined_filter, "combined"),
    ];

    for (filter, filter_name) in &filters {
        let options = Options::default()
            .with_filters(filter.clone())
            .with_processing(Processing::Sequential);

        let shared_options = make_shared(options);

        group.bench_function(format!("sequential_{}", filter_name), |b| {
            self::common::bench_word_tally_with_string(
                b,
                text_sample.clone(),
                &shared_options,
                |text| {
                    let mut temp_file = tempfile::NamedTempFile::new().unwrap();
                    std::io::Write::write_all(&mut temp_file, text.as_bytes()).unwrap();
                    let input = Input::new(temp_file.path(), Io::Buffered).unwrap();
                    (temp_file, input)
                },
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
