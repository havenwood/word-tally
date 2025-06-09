use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use word_tally::{Performance, Threads};

#[test]
fn test_constants() {
    assert_eq!(Performance::base_stdin_tally_capacity(), 1024);
}

#[test]
fn test_with_base_stdin_size() {
    let perf = Performance::default().with_base_stdin_size(512 * 1024);
    assert_eq!(perf.base_stdin_size, 512 * 1024);
}

#[test]
fn test_clone() {
    let perf1 = Performance::default().with_chunk_size(32768);
    let perf2 = perf1;
    assert_eq!(perf1, perf2);
}

#[test]
fn test_partial_eq() {
    let perf1 = Performance::default();
    let perf2 = Performance::default();
    assert_eq!(perf1, perf2);

    let perf3 = Performance::default().with_chunk_size(32768);
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

#[test]
fn test_serde() {
    let perf = Performance::default().with_chunk_size(32768);

    let json = serde_json::to_string(&perf).expect("serialize JSON");
    let deserialized: Performance = serde_json::from_str(&json).expect("deserialize JSON");

    assert_eq!(perf, deserialized);
}

#[test]
fn test_environment_logic() {
    let perf = Performance::from_env();

    // Should get defaults when no env vars are set
    assert_eq!(perf.uniqueness_ratio, 32);
    assert_eq!(perf.words_per_kb, 128);
    assert_eq!(perf.chunk_size, 65536);
    assert_eq!(perf.base_stdin_size, 256 * 1024);
    assert_eq!(perf.threads, Threads::All);
}
