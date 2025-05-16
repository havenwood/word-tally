use word_tally::{Performance, Threads};

#[test]
fn test_parse_env_var_indirectly() {
    let performance = Performance::default();

    assert_eq!(performance.capacity(Some(100_000)), 1940); // (100_000 / 1024 * 200) / 10 = 1940 (with integer division)
}

#[test]
fn test_parallel_mode_configuration() {
    let perf_sequential = Performance::default();
    assert_eq!(perf_sequential.chunk_size, 65_536); // Default value (64KB)

    // Test explicit configuration
    let perf_configured = Performance::default()
        .with_chunk_size(32_768)
        .with_words_per_kb(20);

    assert_eq!(perf_configured.chunk_size, 32_768);
    assert_eq!(perf_configured.words_per_kb, 20);
}

#[test]
fn test_thread_configuration() {
    // Test thread count setting
    let threads_default = Performance::default();
    assert_eq!(threads_default.threads, Threads::All);

    // Test setting specific thread count
    let threads_custom = Performance::default().with_threads(Threads::Count(4));
    assert_eq!(threads_custom.threads, Threads::Count(4));

    // Test thread count conversion
    let threads_from_int: Threads = 8.into();
    assert_eq!(threads_from_int, Threads::Count(8));
}

#[test]
fn test_size_hint_methods() {
    // Test different size hints
    let no_hint_perf = Performance::default();
    assert_eq!(no_hint_perf.capacity(None), 5120); // Default capacity based on BASE_STDIN_TALLY_CAPACITY

    // Test with size hint
    let with_hint = Performance::default();
    assert_eq!(with_hint.capacity(Some(100_000)), 1940); // (100_000 / 1024 * 200) / 10 = 1940 (integer division)
}
