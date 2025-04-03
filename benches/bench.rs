use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use fake::Fake;
use fake::faker::lorem::en::Words;
use std::io::Cursor;
use std::sync::OnceLock;
use word_tally::{Filters, MinChars, MinCount, Options, Sort, WordTally};

static TEXT: OnceLock<String> = OnceLock::new();

fn sample_text() -> &'static str {
    TEXT.get_or_init(|| {
        (0..500).map(|_| Words(10..30).fake::<Vec<String>>().join(" ")).collect::<Vec<_>>().join("\n")
    })
}

fn bench_variant(c: &mut Criterion, group_name: &str, name: &str, setup: impl Fn() -> WordTally) {
    let mut group = c.benchmark_group(group_name);
    group.bench_function(name, |b| {
        b.iter_batched(|| (), |_| setup(), BatchSize::LargeInput);
    });
    group.finish();
}

fn run_benchmarks(c: &mut Criterion) {
    let make_input = || Cursor::new(sample_text());
    let sort_options = [
        ("unsorted", Sort::Unsorted),
        ("ascending", Sort::Asc),
        ("descending", Sort::Desc),
    ];
    for (name, sort) in sort_options {
        bench_variant(c, "sorting", name, || {
            WordTally::new(
                make_input(),
                Options {
                    sort,
                    ..Options::default()
                },
                Filters::default(),
            )
        });
    }

    bench_variant(c, "filtering", "min_count", || {
        WordTally::new(
            make_input(),
            Options::default(),
            Filters {
                min_count: Some(MinCount(2)),
                ..Filters::default()
            },
        )
    });

    bench_variant(c, "filtering", "min_chars", || {
        WordTally::new(
            make_input(),
            Options::default(),
            Filters {
                min_chars: Some(MinChars(3)),
                ..Filters::default()
            },
        )
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .configure_from_args();
    targets = run_benchmarks
}

criterion_main!(benches);
