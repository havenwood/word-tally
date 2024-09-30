use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use std::io::Cursor;
use std::sync::OnceLock;
use word_tally::{Filters, MinChars, Options, Sort, WordTally};

const INPUT: &str = "Orchids bloom silently\nMicrocontrollers hum\nPhalaenopsis thrives\n\
    Data packets route\nPhalaenopsis BLOOM\nDendrobium anchors\nPhotosynthesis proceeds\n\
    Circuit boards and roots\nTranspiration observed\nDendrobium grows\n\
    Algorithms compute\nOrchids in data streams\nPhalaenopsis\nDENDROBIUM\n\
    microcontrollers HUM\nCircuit Boards and ROOTS\ntranspiration OBSERVED\n\
    DATA packets route\nPhalaenopsis BLOOM\nOrchids in DATA streams";

static INPUT_LOCK: OnceLock<String> = OnceLock::new();

fn repeated_input() -> &'static String {
    INPUT_LOCK.get_or_init(|| INPUT.repeat(42))
}

fn prepare_input() -> Cursor<&'static str> {
    let input = repeated_input();
    Cursor::new(input)
}

fn bench_new_unsorted(c: &mut Criterion) {
    c.bench_function("new_unsorted", |b| {
        b.iter_batched(
            prepare_input,
            |input| {
                WordTally::new(
                    input,
                    Options {
                        sort: Sort::Unsorted,
                        ..Options::default()
                    },
                    Filters::default(),
                )
            },
            BatchSize::SmallInput,
        );
    });
}

fn bench_new_sorted(c: &mut Criterion) {
    c.bench_function("new_sorted", |b| {
        b.iter_batched(
            prepare_input,
            |input| {
                WordTally::new(
                    input,
                    Options {
                        sort: Sort::Asc,
                        ..Options::default()
                    },
                    Filters::default(),
                )
            },
            BatchSize::SmallInput,
        );
    });
}

fn bench_new_min_chars(c: &mut Criterion) {
    c.bench_function("new_min_chars", |b| {
        b.iter_batched(
            prepare_input,
            |input| {
                WordTally::new(
                    input,
                    Options {
                        sort: Sort::Unsorted,
                        ..Options::default()
                    },
                    Filters {
                        min_chars: Some(MinChars(5)),
                        ..Filters::default()
                    },
                )
            },
            BatchSize::SmallInput,
        );
    });
}

fn bench_new_min_count(c: &mut Criterion) {
    c.bench_function("new_min_count", |b| {
        b.iter_batched(
            prepare_input,
            |input| {
                WordTally::new(
                    input,
                    Options {
                        sort: Sort::Unsorted,
                        ..Options::default()
                    },
                    Filters::default(),
                )
            },
            BatchSize::SmallInput,
        );
    });
}

fn bench_sort(c: &mut Criterion) {
    c.bench_function("sort", |b| {
        b.iter_batched(
            || {
                WordTally::new(
                    prepare_input(),
                    Options {
                        sort: Sort::Unsorted,
                        ..Options::default()
                    },
                    Filters::default(),
                )
            },
            |mut tally| tally.sort(Sort::Asc),
            BatchSize::SmallInput,
        );
    });
}

fn configure_criterion() -> Criterion {
    Criterion::default().noise_threshold(0.1)
}

criterion_group! {
    name = benches;
    config = configure_criterion();
    targets = bench_new_unsorted, bench_new_sorted, bench_new_min_chars, bench_new_min_count, bench_sort
}
criterion_main!(benches);
