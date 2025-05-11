//! Benchmarks comparing sequential vs parallel processing strategies.

use criterion::{Criterion, criterion_group, criterion_main};
use word_tally::{Filters, Input, Io, Options, Processing};

#[path = "common.rs"]
pub mod common;
use self::common::{
    create_bench_group, make_shared, medium_text, small_text, standard_criterion_config,
};

/// Benchmark comparing sequential vs parallel processing strategies for different text sizes
fn bench_processing_comparison(c: &mut Criterion) {
    let mut group = create_bench_group(c, "processing");

    let processing_strategies = [
        (Processing::Sequential, "sequential"),
        (Processing::Parallel, "parallel"),
    ];

    // Prepare text samples
    let small_text_sample = small_text();
    let medium_text_sample = medium_text();

    // Small text benchmark (measures processing overhead)
    for (processing, name) in &processing_strategies {
        let options = Options::default().with_processing(*processing);
        let shared_options = make_shared(options);

        group.bench_function(format!("small_{}", name), |b| {
            self::common::bench_word_tally_with_string(
                b,
                small_text_sample.clone(),
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

    // Medium text benchmark (better shows parallel benefits)
    for (processing, name) in &processing_strategies {
        let options = Options::default().with_processing(*processing);
        let shared_options = make_shared(options);

        group.bench_function(format!("medium_{}", name), |b| {
            self::common::bench_word_tally_with_string(
                b,
                medium_text_sample.clone(),
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

/// Benchmark regex pattern performance
fn bench_regex_patterns(c: &mut Criterion) {
    let mut group = create_bench_group(c, "regex_patterns");
    let text_sample = medium_text();

    let base_options = Options::default()
        .with_processing(Processing::Parallel)
        .with_filters(Filters::default());

    group.bench_function("no_patterns", |b| {
        self::common::bench_word_tally_with_string(
            b,
            text_sample.clone(),
            &make_shared(base_options.clone()),
            |text| {
                let mut temp_file = tempfile::NamedTempFile::new().unwrap();
                std::io::Write::write_all(&mut temp_file, text.as_bytes()).unwrap();
                let input = Input::new(temp_file.path(), Io::Buffered).unwrap();
                (temp_file, input)
            },
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
            .unwrap(),
    );

    group.bench_function("few_patterns", |b| {
        self::common::bench_word_tally_with_string(
            b,
            text_sample.clone(),
            &make_shared(few_patterns_options.clone()),
            |text| {
                let mut temp_file = tempfile::NamedTempFile::new().unwrap();
                std::io::Write::write_all(&mut temp_file, text.as_bytes()).unwrap();
                let input = Input::new(temp_file.path(), Io::Buffered).unwrap();
                (temp_file, input)
            },
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
            .unwrap(),
    );

    group.bench_function("many_patterns", |b| {
        self::common::bench_word_tally_with_string(
            b,
            text_sample.clone(),
            &make_shared(many_patterns_options.clone()),
            |text| {
                let mut temp_file = tempfile::NamedTempFile::new().unwrap();
                std::io::Write::write_all(&mut temp_file, text.as_bytes()).unwrap();
                let input = Input::new(temp_file.path(), Io::Buffered).unwrap();
                (temp_file, input)
            },
        );
    });

    group.finish();
}

fn run_benchmarks(c: &mut Criterion) {
    bench_processing_comparison(c);
    bench_regex_patterns(c);
}

criterion_group! {
    name = benches;
    config = standard_criterion_config();
    targets = run_benchmarks
}

criterion_main!(benches);
