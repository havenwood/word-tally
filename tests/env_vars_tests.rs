use word_tally::{Io, Performance, Processing, SizeHint, Threads};

#[test]
fn test_parse_env_var_indirectly() {
    let size_hint = SizeHint::Bytes(100_000);

    let performance = Performance::default()
        .with_processing(Processing::Parallel)
        .with_size_hint(size_hint);

    assert_eq!(performance.estimate_capacity(), 10_000); // 100_000 / 10 (default uniqueness ratio)
}

#[test]
fn test_parallel_mode_configuration() {
    let perf_sequential = Performance::default();
    assert_eq!(perf_sequential.processing(), Processing::Sequential);
    assert_eq!(perf_sequential.io(), Io::Streamed);
    assert_eq!(perf_sequential.chunk_size(), 65_536); // Default value (64KB)

    // Switching to parallel should retain the configuration
    let perf_parallel = perf_sequential.with_processing(Processing::Parallel);
    assert_eq!(perf_parallel.processing(), Processing::Parallel);
    assert_eq!(perf_parallel.chunk_size(), 65_536);

    // Test explicit configuration
    let perf_configured = Performance::default()
        .with_chunk_size(32_768)
        .with_word_density(20)
        .with_processing(Processing::Parallel)
        .with_io(Io::MemoryMapped);

    assert_eq!(perf_configured.chunk_size(), 32_768);
    assert_eq!(perf_configured.unique_word_density(), 20);
}

#[test]
fn test_thread_configuration() {
    // Test thread count setting
    let threads_default = Performance::default();
    assert_eq!(threads_default.threads(), Threads::All);

    // Test setting specific thread count
    let threads_custom = Performance::default().with_threads(Threads::Count(4));
    assert_eq!(threads_custom.threads(), Threads::Count(4));

    // Test thread count conversion
    let threads_from_int: Threads = 8.into();
    assert_eq!(threads_from_int, Threads::Count(8));
}

#[test]
fn test_size_hint_methods() {
    // Test different size hints
    let no_hint_perf = Performance::default();
    assert_eq!(no_hint_perf.size_hint(), SizeHint::None);
    assert_eq!(no_hint_perf.estimate_capacity(), 1024); // Default capacity

    // Test with size hint
    let size_hint = SizeHint::Bytes(100_000);
    let with_hint = Performance::default().with_size_hint(size_hint);
    assert_eq!(with_hint.estimate_capacity(), 10_000); // 100_000 / 10

    // Test chunk capacity estimation with new formula: (chunk_size * word_density) / 10 + 10
    assert_eq!(with_hint.estimate_chunk_capacity(1000), 1510); // (1000 * 15) / 10 + 10
}
