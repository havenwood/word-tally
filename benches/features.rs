//! Benchmarks comparing sequential vs parallel processing strategies.

use std::io::Cursor;

use criterion::{Criterion, criterion_group, criterion_main};
use word_tally::{Options, Processing};

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
                Cursor::new,
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
                Cursor::new,
            );
        });
    }

    group.finish();
}

fn run_benchmarks(c: &mut Criterion) {
    bench_processing_comparison(c);
}

criterion_group! {
    name = benches;
    config = standard_criterion_config();
    targets = run_benchmarks
}

criterion_main!(benches);
