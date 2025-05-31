//! Feature benchmarks.

use criterion::{Criterion, criterion_group, criterion_main};
use word_tally::options::encoding::Encoding;
use word_tally::{Filters, Io, Options};

#[path = "common.rs"]
pub mod common;
use self::common::{
    create_bench_group, create_temp_input, make_shared, medium_text, small_text,
    standard_criterion_config,
};

/// Benchmark sequential vs parallel processing
fn bench_processing_comparison(c: &mut Criterion) {
    let mut group = create_bench_group(c, "features/processing_comparison");

    let text_samples = [("small", small_text()), ("medium", medium_text())];

    for (size_name, text_sample) in &text_samples {
        // Test with default (parallel) processing
        let options = Options::default();
        let shared_options = make_shared(options);

        group.bench_function(format!("{size_name}_parallel"), |b| {
            self::common::bench_word_tally_with_string(
                b,
                text_sample,
                &shared_options,
                create_temp_input,
            );
        });

        // Test with sequential processing
        let options = Options::default().with_io(Io::Stream);
        let shared_options = make_shared(options);

        group.bench_function(format!("{size_name}_sequential"), |b| {
            self::common::bench_word_tally_with_string(
                b,
                text_sample,
                &shared_options,
                create_temp_input,
            );
        });
    }

    group.finish();
}

/// Benchmark regex patterns
fn bench_regex_patterns(c: &mut Criterion) {
    let mut group = create_bench_group(c, "features/regex_patterns");
    let text_sample = medium_text();

    let base_options = Options::default().with_filters(Filters::default());

    group.bench_function("patterns_0", |b| {
        self::common::bench_word_tally_with_string(
            b,
            &text_sample,
            &make_shared(base_options.clone()),
            create_temp_input,
        );
    });

    let few_patterns = vec![
        "^[aeiou].*".to_string(),
        ".*ing$".to_string(),
        "^[A-Z][a-z]*$".to_string(),
        "^[a-z]{1,3}$".to_string(),
    ];

    let few_patterns_options = base_options.clone().with_filters(
        Filters::default()
            .with_include_patterns(&few_patterns)
            .expect("compile filter regex"),
    );

    group.bench_function("patterns_4", |b| {
        self::common::bench_word_tally_with_string(
            b,
            &text_sample,
            &make_shared(few_patterns_options.clone()),
            create_temp_input,
        );
    });

    let many_patterns = vec![
        "^[aeiou].*".to_string(),
        ".*ing$".to_string(),
        "^[A-Z][a-z]*$".to_string(),
        "^[a-z]{1,3}$".to_string(),
        "^[^aeiou]*[aeiou][^aeiou]*$".to_string(),
        ".*[0-9].*".to_string(),
        "^(re|un|in|dis).*".to_string(),
        ".*[^a-zA-Z0-9].*".to_string(),
        "^(the|and|but|or|for|with)$".to_string(),
        "^.{10,}$".to_string(),
    ];

    let many_patterns_options = base_options.clone().with_filters(
        Filters::default()
            .with_include_patterns(&many_patterns)
            .expect("compile filter regex"),
    );

    group.bench_function("patterns_10", |b| {
        self::common::bench_word_tally_with_string(
            b,
            &text_sample,
            &make_shared(many_patterns_options.clone()),
            create_temp_input,
        );
    });

    group.finish();
}

/// Benchmark encoding strategies (Unicode vs ASCII)
fn bench_encoding_comparison(c: &mut Criterion) {
    let mut group = create_bench_group(c, "features/encoding_comparison");

    let encoding_strategies = [(Encoding::Unicode, "unicode"), (Encoding::Ascii, "ascii")];

    let text_samples = [("small", small_text()), ("medium", medium_text())];

    for (size_name, text_sample) in &text_samples {
        for (encoding, enc_name) in &encoding_strategies {
            // Test with default (parallel) processing
            let options = Options::default().with_encoding(*encoding);
            let shared_options = make_shared(options);

            group.bench_function(format!("{size_name}_{enc_name}_parallel"), |b| {
                self::common::bench_word_tally_with_string(
                    b,
                    text_sample,
                    &shared_options,
                    create_temp_input,
                );
            });

            // Test with sequential processing (no-parallel)
            let options = Options::default()
                .with_encoding(*encoding)
                .with_io(Io::Stream);
            let shared_options = make_shared(options);

            group.bench_function(format!("{size_name}_{enc_name}_sequential"), |b| {
                self::common::bench_word_tally_with_string(
                    b,
                    text_sample,
                    &shared_options,
                    create_temp_input,
                );
            });
        }
    }

    group.finish();
}

fn run_benchmarks(c: &mut Criterion) {
    bench_processing_comparison(c);
    bench_regex_patterns(c);
    bench_encoding_comparison(c);
}

criterion_group! {
    name = benches;
    config = standard_criterion_config();
    targets = run_benchmarks
}

criterion_main!(benches);
