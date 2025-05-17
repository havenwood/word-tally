use word_tally::Threads;

#[test]
fn test_threads_count() {
    // Test Count variant
    let threads_4 = Threads::Count(4);
    assert_eq!(threads_4.count(), 4);

    let threads_8 = Threads::Count(8);
    assert_eq!(threads_8.count(), 8);

    // Test All variant - should return actual thread count
    let threads_all = Threads::All;
    let all_count = threads_all.count();
    assert!(all_count > 0);
    assert!(all_count <= std::thread::available_parallelism().unwrap().get());
}

#[test]
fn test_threads_default() {
    let default_threads = Threads::default();
    assert_eq!(default_threads, Threads::All);
}

#[test]
fn test_threads_init_pool() {
    // The global thread pool can only be initialized once across all tests
    // So we test that the function exists and returns ok for reasonable inputs
    // Note: Some tests may have already initialized the pool

    // Test with All variant - this should always succeed
    let threads_all = Threads::All;
    let result = threads_all.init_pool();
    // It's OK for this to return either success (first init) or success (already initialized)
    assert!(result.is_ok());

    // Test with specific count
    let threads_2 = Threads::Count(2);
    let result = threads_2.init_pool();
    // This might fail if another test already initialized with a different count
    // but we'll just check that it doesn't panic
    drop(result); // ignore the result as it depends on test execution order
}

#[test]
fn test_threads_from_u16() {
    let threads: Threads = 6u16.into();
    assert_eq!(threads, Threads::Count(6));
    assert_eq!(threads.count(), 6);
}

#[test]
fn test_threads_display() {
    let threads_3 = Threads::Count(3);
    assert_eq!(format!("{}", threads_3), "3");

    let threads_all = Threads::All;
    let display = format!("{}", threads_all);
    assert!(display.parse::<usize>().is_ok());
}

#[test]
fn test_threads_edge_cases() {
    // Test with 1 thread
    let threads_1 = Threads::Count(1);
    assert_eq!(threads_1.count(), 1);
    // Note: init_pool should only be called once globally, so we don't test it here
    // since other tests may have already initialized the pool

    // Test with 0 threads (edge case, though not recommended)
    let threads_0 = Threads::Count(0);
    assert_eq!(threads_0.count(), 0);
    // Note: init_pool with 0 threads may fail, but we're testing the count() method
}

#[test]
fn test_threads_traits() {
    let threads_a = Threads::Count(4);
    let threads_b = Threads::Count(4);
    let threads_c = Threads::Count(8);

    // Test PartialEq
    assert_eq!(threads_a, threads_b);
    assert_ne!(threads_a, threads_c);

    // Test Clone
    let threads_clone = threads_a;
    assert_eq!(threads_a, threads_clone);

    // Test Ord
    assert!(threads_a < threads_c);
    assert!(threads_c > threads_a);
    // The ordering depends on the enum discriminant order
    // All comes before Count in the enum definition
}

#[test]
fn test_threads_serialization() {
    // Test serialization
    let threads = Threads::Count(4);
    let json = serde_json::to_string(&threads).unwrap();
    assert_eq!(json, r#"{"Count":4}"#);

    let threads_all = Threads::All;
    let json_all = serde_json::to_string(&threads_all).unwrap();
    assert_eq!(json_all, r#""All""#);

    // Test deserialization
    let deserialized: Threads = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, threads);

    let deserialized_all: Threads = serde_json::from_str(&json_all).unwrap();
    assert_eq!(deserialized_all, threads_all);
}
