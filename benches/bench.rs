use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use fake::Fake;
use fake::faker::lorem::en::Words;
use std::io::Cursor;
use std::sync::{Arc, OnceLock};
use word_tally::{Concurrency, Filters, Options, SizeHint, Sort, WordTally};

static TEXT: OnceLock<String> = OnceLock::new();

fn sample_text() -> &'static str {
    TEXT.get_or_init(|| {
        (0..2500)
            .map(|_| Words(8..15).fake::<Vec<String>>().join(" "))
            .collect::<Vec<_>>()
            .join("\n")
    })
}

// Create a shared reference for reusing values in benchmarks
fn make_shared<T>(value: T) -> Arc<T> {
    Arc::new(value)
}

fn bench_variant(
    c: &mut Criterion,
    group_name: &str,
    name: &str,
    sort: Sort,
    filters: Filters,
    parallel: bool,
) {
    // Create a static reference to options that we'll reuse
    let concurrency = if parallel {
        Concurrency::Parallel
    } else {
        Concurrency::Sequential
    };

    // Create options with sorting, filtering and performance configuration
    let options = Options::default()
        .with_sort(sort)
        .with_filters(filters)
        .with_concurrency(concurrency)
        .with_size_hint(SizeHint::default());

    // Create a shared reference for the options
    let shared_options = make_shared(options);

    c.benchmark_group(group_name).bench_function(name, |b| {
        b.iter_batched(
            || Cursor::new(sample_text()),
            |input| WordTally::new(input, &shared_options),
            BatchSize::LargeInput,
        );
    });
}

fn run_benchmarks(c: &mut Criterion) {
    let sort_options = [
        ("unsorted", Sort::Unsorted),
        ("ascending", Sort::Asc),
        ("descending", Sort::Desc),
    ];

    for (name, sort) in sort_options {
        // Sequential sorting benchmarks
        bench_variant(c, "sorting", name, sort, Filters::default(), false);

        // Parallel sorting benchmarks
        bench_variant(c, "sorting_parallel", name, sort, Filters::default(), true);
    }

    let min_count_filter = Filters::default().with_min_count(2);
    let min_chars_filter = Filters::default().with_min_chars(3);

    // Filtering benchmarks
    bench_variant(
        c,
        "filtering",
        "min_count",
        Sort::default(),
        min_count_filter,
        false,
    );

    bench_variant(
        c,
        "filtering",
        "min_chars",
        Sort::default(),
        min_chars_filter,
        false,
    );
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .configure_from_args()
        .sample_size(10);
    targets = run_benchmarks
}

criterion_main!(benches);
