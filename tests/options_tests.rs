use word_tally::{
    Case, Filters, Format, Io, Options, Performance, Processing, Serialization, Sort, Threads,
};

#[test]
fn test_options_format_default() {
    let options = Options::default();
    assert_eq!(options.serialization().format(), Format::Text);
}

#[test]
fn test_options_with_format() {
    let options = Options::default().with_format(Format::Json);
    assert_eq!(options.serialization().format(), Format::Json);

    let options = Options::default()
        .with_format(Format::Csv)
        .with_case(Case::Upper);

    assert_eq!(options.serialization().format(), Format::Csv);
    assert_eq!(options.case(), Case::Upper);
}

#[test]
fn test_options_display_includes_format() {
    let options = Options::default().with_format(Format::Json);
    let display_string = options.to_string();

    assert!(display_string.contains("serialization:"));
    assert!(display_string.contains("format: json"));
}

#[test]
fn test_format_field_in_struct() {
    let options = Options::default();
    assert_eq!(options.serialization().format(), Format::Text);

    let options2 = Options::default().with_format(Format::Json);
    assert_eq!(options2.serialization().format(), Format::Json);
}

#[test]
fn test_format_serialization() {
    let options = Options::default().with_format(Format::Json);
    let serialized = serde_json::to_string(&options).expect("serialize JSON");

    assert!(serialized.contains("\"format\":\"Json\""));

    let deserialized: Options = serde_json::from_str(&serialized).expect("deserialize JSON");
    assert_eq!(deserialized.serialization().format(), Format::Json);
}

#[test]
fn test_comprehensive_options_serialization() {
    let options = Options::default()
        .with_case(Case::Upper)
        .with_sort(Sort::Desc)
        .with_format(Format::Json)
        .with_io(Io::MemoryMapped)
        .with_filters(Filters::default().with_min_chars(3).with_min_count(2));

    let json = serde_json::to_string(&options).expect("serialize JSON");

    let deserialized: Options = serde_json::from_str(&json).expect("deserialize JSON");

    assert_eq!(options.case(), deserialized.case());
    assert_eq!(options.sort(), deserialized.sort());
    assert_eq!(options.io(), deserialized.io());
    assert_eq!(
        options.filters().min_chars(),
        deserialized.filters().min_chars()
    );
    assert_eq!(
        options.filters().min_count(),
        deserialized.filters().min_count()
    );
    assert_eq!(
        options.serialization().format(),
        deserialized.serialization().format()
    );
}

#[test]
fn test_options_new() {
    let case = Case::Upper;
    let sort = Sort::Desc;
    let serialization = Serialization::default();
    let filters = Filters::default();
    let io = Io::Buffered;
    let processing = Processing::Parallel;
    let performance = Performance::default();

    let options = Options::new(
        case,
        sort,
        serialization,
        filters,
        io,
        processing,
        performance,
    );

    assert_eq!(options.case(), case);
    assert_eq!(options.sort(), sort);
    assert_eq!(options.io(), io);
    assert_eq!(options.processing(), processing);
}

#[test]
fn test_options_new_with_default_filters() {
    let case = Case::Lower;
    let sort = Sort::Asc;
    let serialization = Serialization::default();
    let io = Io::MemoryMapped;
    let processing = Processing::Sequential;
    let performance = Performance::default();

    let options = Options::new(
        case,
        sort,
        serialization,
        Filters::default(),
        io,
        processing,
        performance,
    );

    assert_eq!(options.case(), case);
    assert_eq!(options.sort(), sort);
    assert_eq!(options.io(), io);
    assert_eq!(options.processing(), processing);
    assert!(matches!(options.filters(), _));
}

#[test]
fn test_with_case() {
    let options = Options::default().with_case(Case::Upper);
    assert_eq!(options.case(), Case::Upper);
}

#[test]
fn test_with_sort() {
    let options = Options::default().with_sort(Sort::Desc);
    assert_eq!(options.sort(), Sort::Desc);
}

#[test]
fn test_with_serialization() {
    let serialization =
        Serialization::new(Format::Json, ",").expect("should create valid serialization");
    let options = Options::default().with_serialization(serialization);
    assert_eq!(options.serialization().format(), Format::Json);
}

#[test]
fn test_with_filters() {
    let filters = Filters::default();
    let options = Options::default().with_filters(filters);
    assert_eq!(options.filters().min_chars(), None);
}

#[test]
fn test_with_performance() {
    let performance = Performance::default().with_threads(Threads::Count(4));
    let options = Options::default().with_performance(performance);
    assert_eq!(options.performance().threads.count(), 4);
}

#[test]
fn test_with_delimiter() {
    let options = Options::default().with_delimiter("::".to_string());
    assert_eq!(options.serialization().delimiter(), "::");
}

#[test]
fn test_with_io() {
    let options = Options::default().with_io(Io::MemoryMapped);
    assert_eq!(options.io(), Io::MemoryMapped);
}

#[test]
fn test_with_processing() {
    let options = Options::default().with_processing(Processing::Parallel);
    assert_eq!(options.processing(), Processing::Parallel);
}

#[test]
fn test_with_threads() {
    let options = Options::default().with_threads(Threads::Count(8));
    assert_eq!(options.performance().threads.count(), 8);
}

#[test]
fn test_with_uniqueness_ratio() {
    let options = Options::default().with_uniqueness_ratio(75);
    assert_eq!(options.performance().uniqueness_ratio, 75);
}

#[test]
fn test_with_words_per_kb() {
    let options = Options::default().with_words_per_kb(120);
    assert_eq!(options.performance().words_per_kb, 120);
}

#[test]
fn test_with_chunk_size() {
    let options = Options::default().with_chunk_size(8192);
    assert_eq!(options.performance().chunk_size, 8192);
}

#[test]
fn test_getters() {
    let options = Options::default()
        .with_case(Case::Lower)
        .with_sort(Sort::Asc)
        .with_io(Io::Streamed)
        .with_processing(Processing::Parallel);

    assert_eq!(options.case(), Case::Lower);
    assert_eq!(options.sort(), Sort::Asc);
    assert_eq!(options.serialization().format(), Format::Text);
    assert!(matches!(options.filters(), _));
    assert!(matches!(options.performance(), _));
    assert_eq!(options.io(), Io::Streamed);
    assert_eq!(options.processing(), Processing::Parallel);
}

#[test]
fn test_builder_chaining() {
    let options = Options::default()
        .with_case(Case::Upper)
        .with_sort(Sort::Desc)
        .with_format(Format::Json)
        .with_delimiter("||".to_string())
        .with_io(Io::Buffered)
        .with_processing(Processing::Parallel)
        .with_threads(Threads::Count(4))
        .with_uniqueness_ratio(80)
        .with_words_per_kb(150)
        .with_chunk_size(16384);

    assert_eq!(options.case(), Case::Upper);
    assert_eq!(options.sort(), Sort::Desc);
    assert_eq!(options.serialization().format(), Format::Json);
    assert_eq!(options.serialization().delimiter(), "||");
    assert_eq!(options.io(), Io::Buffered);
    assert_eq!(options.processing(), Processing::Parallel);
    assert_eq!(options.performance().threads.count(), 4);
    assert_eq!(options.performance().uniqueness_ratio, 80);
    assert_eq!(options.performance().words_per_kb, 150);
    assert_eq!(options.performance().chunk_size, 16384);
}

#[test]
fn test_as_ref_serialization() {
    let options = Options::default().with_format(Format::Csv);
    let serialization_ref: &Serialization = options.as_ref();
    assert_eq!(serialization_ref.format(), Format::Csv);
}

#[test]
fn test_as_ref_filters() {
    let options = Options::default();
    let filters_ref: &Filters = options.as_ref();
    assert_eq!(filters_ref.min_chars(), None);
}

#[test]
fn test_as_ref_performance() {
    let options = Options::default().with_threads(Threads::Count(2));
    let performance_ref: &Performance = options.as_ref();
    assert_eq!(performance_ref.threads.count(), 2);
}

#[test]
fn test_options_serde_full() {
    let options = Options::default()
        .with_case(Case::Upper)
        .with_sort(Sort::Desc)
        .with_format(Format::Json)
        .with_io(Io::Buffered)
        .with_processing(Processing::Parallel);

    let serialized = serde_json::to_string(&options).expect("serialize JSON");
    let deserialized: Options = serde_json::from_str(&serialized).expect("deserialize JSON");

    assert_eq!(deserialized.case(), options.case());
    assert_eq!(deserialized.sort(), options.sort());
    assert_eq!(
        deserialized.serialization().format(),
        options.serialization().format()
    );
    assert_eq!(deserialized.io(), options.io());
    assert_eq!(deserialized.processing(), options.processing());
}
