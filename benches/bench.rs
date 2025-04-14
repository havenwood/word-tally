use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use fake::Fake;
use fake::faker::lorem::en::Words;
use std::io::Cursor;
use std::sync::OnceLock;
use word_tally::{Concurrency, Config, Filters, MinChars, MinCount, Options, SizeHint, Sort, WordTally};

static TEXT: OnceLock<String> = OnceLock::new();

fn sample_text() -> &'static str {
    TEXT.get_or_init(|| {
        (0..2500)
            .map(|_| Words(8..15).fake::<Vec<String>>().join(" "))
            .collect::<Vec<_>>()
            .join("\n")
    })
}

fn bench_variant(c: &mut Criterion, group_name: &str, name: &str, setup: impl Fn() -> WordTally) {
    c.benchmark_group(group_name)
        .bench_function(name, |b| {
            b.iter_batched(|| (), |_| setup(), BatchSize::LargeInput);
        });
}

/// Create a WordTally creation function for benchmarking with consistent parameters
fn create_tally(
    input: impl Fn() -> Cursor<&'static str>,
    sort: Sort,
    filters: Filters,
    parallel: bool
) -> impl Fn() -> WordTally {
    let options = Options::with_sort(sort);
    let concurrency = if parallel { Concurrency::Parallel } else { Concurrency::Sequential };
    move || {
        let config = Config::default()
            .with_concurrency(concurrency)
            .with_size_hint(SizeHint::default());
        WordTally::new(input(), options, filters.clone(), config)
    }
}

fn run_benchmarks(c: &mut Criterion) {
    let make_input = || Cursor::new(sample_text());
    let sort_options = [
        ("unsorted", Sort::Unsorted),
        ("ascending", Sort::Asc),
        ("descending", Sort::Desc),
    ];

    for (name, sort) in sort_options {
        bench_variant(c, "sorting", name,
            create_tally(make_input, sort, Filters::default(), false));

        bench_variant(c, "sorting_parallel", name,
            create_tally(make_input, sort, Filters::default(), true));
    }

    let min_count_filter = Filters {
        min_count: Some(MinCount(2)),
        ..Filters::default()
    };

    let min_chars_filter = Filters {
        min_chars: Some(MinChars(3)),
        ..Filters::default()
    };

    bench_variant(c, "filtering", "min_count",
        create_tally(make_input, Sort::default(), min_count_filter, false));

    bench_variant(c, "filtering", "min_chars",
        create_tally(make_input, Sort::default(), min_chars_filter, false));
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .configure_from_args()
        .sample_size(10);
    targets = run_benchmarks
}

criterion_main!(benches);
