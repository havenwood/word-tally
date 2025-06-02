//! Feature benchmarks.

use criterion::{Criterion, criterion_group, criterion_main};
use word_tally::options::encoding::Encoding;
use word_tally::{Io, Options};

#[path = "common.rs"]
pub mod common;
use self::common::{
    create_bench_group, create_temp_input, make_shared, medium_text, standard_criterion_config,
};

/// Benchmark sequential vs parallel processing
fn bench_processing_comparison(c: &mut Criterion) {
    let mut group = create_bench_group(c, "features/processing_comparison");
    let text_sample = medium_text();

    // Test with default (parallel) processing
    let options = Options::default();
    let shared_options = make_shared(options);

    group.bench_function("parallel", |b| {
        common::bench_word_tally_with_string(b, &text_sample, &shared_options, create_temp_input);
    });

    // Test with sequential processing
    let options = Options::default().with_io(Io::Stream);
    let shared_options = make_shared(options);

    group.bench_function("sequential", |b| {
        common::bench_word_tally_with_string(b, &text_sample, &shared_options, create_temp_input);
    });

    group.finish();
}

/// Benchmark encoding strategies (Unicode vs ASCII)
fn bench_encoding_comparison(c: &mut Criterion) {
    let mut group = create_bench_group(c, "features/encoding_comparison");
    let text_sample = medium_text();

    let encoding_strategies = [(Encoding::Unicode, "unicode"), (Encoding::Ascii, "ascii")];

    for (encoding, enc_name) in &encoding_strategies {
        let options = Options::default().with_encoding(*encoding);
        let shared_options = make_shared(options);

        group.bench_function(*enc_name, |b| {
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
    bench_processing_comparison(c);
    bench_encoding_comparison(c);
}

criterion_group! {
    name = benches;
    config = standard_criterion_config();
    targets = run_benchmarks
}

criterion_main!(benches);
