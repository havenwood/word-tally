use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use once_cell::sync::Lazy;
use std::io::Cursor;
use word_tally::{Case, Minimums, Sort, WordTally};

const BASE_INPUT: &str = "Orchids bloom silently\nMicrocontrollers hum\nPhalaenopsis thrives\n\
    Data packets route\nPhalaenopsis BLOOM\nDendrobium anchors\nPhotosynthesis proceeds\n\
    Circuit boards and roots\nTranspiration observed\nDendrobium grows\n\
    Algorithms compute\nOrchids in data streams\nPhalaenopsis\nDENDROBIUM\n\
    microcontrollers HUM\nCircuit Boards and ROOTS\ntranspiration OBSERVED\n\
    DATA packets route\nPhalaenopsis BLOOM\nOrchids in DATA streams";

static BENCHMARK_INPUT: Lazy<String> = Lazy::new(|| BASE_INPUT.repeat(42));

fn prepare_input() -> Cursor<&'static str> {
    Cursor::new(&BENCHMARK_INPUT)
}

fn bench_new_unsorted(c: &mut Criterion) {
    c.bench_function("new_unsorted", |b| {
        b.iter_batched(
            prepare_input,
            |input| WordTally::new(input, Case::Lower, Sort::Unsorted, Minimums::default()),
            BatchSize::SmallInput,
        );
    });
}

fn bench_new_sorted(c: &mut Criterion) {
    c.bench_function("new_sorted", |b| {
        b.iter_batched(
            prepare_input,
            |input| WordTally::new(input, Case::Lower, Sort::Asc, Minimums::default()),
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
                    Case::Lower,
                    Sort::Unsorted,
                    Minimums {
                        chars: 5,
                        ..Minimums::default()
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
                    Case::Lower,
                    Sort::Unsorted,
                    Minimums {
                        count: 2,
                        ..Minimums::default()
                    },
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
                    Case::Lower,
                    Sort::Unsorted,
                    Minimums::default(),
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
