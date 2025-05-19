use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use word_tally::{Performance, Threads};

// Test constants
#[test]
fn test_constants() {
    // These values are documented in the Performance struct
    assert_eq!(Performance::base_stdin_tally_capacity(), 128);
}

// Test builder methods
#[test]
fn test_with_base_stdin_size() {
    let perf = Performance::default().with_base_stdin_size(512 * 1024);
    assert_eq!(perf.base_stdin_size, 512 * 1024);
}

#[test]
fn test_with_verbose() {
    let perf = Performance::default().with_verbose(true);
    assert!(perf.verbose);

    let perf = Performance::default().with_verbose(false);
    assert!(!perf.verbose);
}

// Test calculation methods
#[test]
fn test_lines_per_chunk() {
    let perf = Performance::default();
    assert_eq!(perf.lines_per_chunk(), 819); // 65536 / 80

    // Test with small chunk size to hit the minimum
    let perf_small = Performance::default().with_chunk_size(1024);
    assert_eq!(perf_small.lines_per_chunk(), 128); // minimum value
}

// Test capacity per thread
#[test]
fn test_capacity_per_thread() {
    // Test with single thread
    let perf = Performance::default().with_threads(Threads::Count(1));
    assert_eq!(perf.capacity_per_thread(), 128); // full capacity

    // Test with multiple threads
    let perf_multi = Performance::default().with_threads(Threads::Count(10));
    assert_eq!(perf_multi.capacity_per_thread(), 128); // hits base capacity limit
}

// Test trait implementations
#[test]
fn test_clone() {
    let perf1 = Performance::default()
        .with_verbose(true)
        .with_chunk_size(32768);
    let perf2 = perf1;
    assert_eq!(perf1, perf2);
}

#[test]
fn test_partial_eq() {
    let perf1 = Performance::default();
    let perf2 = Performance::default();
    assert_eq!(perf1, perf2);

    let perf3 = Performance::default().with_verbose(true);
    assert_ne!(perf1, perf3);
}

#[test]
fn test_ord() {
    let perf1 = Performance::default().with_uniqueness_ratio(5);
    let perf2 = Performance::default().with_uniqueness_ratio(10);
    assert!(perf1 < perf2);
}

#[test]
fn test_hash() {
    let perf1 = Performance::default();
    let perf2 = Performance::default();

    let mut hasher1 = DefaultHasher::new();
    let mut hasher2 = DefaultHasher::new();

    perf1.hash(&mut hasher1);
    perf2.hash(&mut hasher2);

    assert_eq!(hasher1.finish(), hasher2.finish());
}

// Test serialization
#[test]
fn test_serde() {
    let perf = Performance::default()
        .with_verbose(true)
        .with_chunk_size(32768);

    let json = serde_json::to_string(&perf).expect("serialize JSON");
    let deserialized: Performance = serde_json::from_str(&json).expect("deserialize JSON");

    assert_eq!(perf, deserialized);
}

// Test environment variable parsing simulation
#[test]
fn test_environment_logic() {
    // Test the logic without actually setting environment variables
    // This tests the default behavior when no env vars are set
    let perf = Performance::from_env();

    // Should get defaults when no env vars are set
    assert_eq!(perf.uniqueness_ratio, 256);
    assert_eq!(perf.words_per_kb, 128);
    assert_eq!(perf.chunk_size, 65536);
    assert_eq!(perf.base_stdin_size, 256 * 1024);
    assert!(!perf.verbose);
    assert_eq!(perf.threads, Threads::All);
}
