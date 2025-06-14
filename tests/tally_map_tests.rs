//! Tests for `TallyMap` functionality

use core::convert::AsRef;

use word_tally::{
    Case, Count, Mapped, Options, TallyMap, Word,
    options::{encoding::Encoding, io::Io},
};

// Helper to create TallyMap from word pairs
fn make_tally(words: &[(&str, usize)]) -> TallyMap {
    let mut tally = TallyMap::new();
    for (word, count) in words {
        for _ in 0..*count {
            tally
                .add_words(word, Case::Original, Encoding::Unicode)
                .expect("unicode segmentation should not fail");
        }
    }
    tally
}

#[test]
fn test_new() {
    let tally = TallyMap::new();
    assert!(tally.is_empty());
    assert_eq!(tally.len(), 0);
}

#[test]
fn test_with_capacity() {
    let tally = TallyMap::with_capacity(100);
    assert!(tally.is_empty());
    assert_eq!(tally.len(), 0);
}

#[test]
fn test_reserve() {
    let mut tally = TallyMap::new();
    tally.reserve(50);
    assert!(tally.is_empty());
}

#[test]
fn test_is_empty() {
    let mut tally = TallyMap::new();
    assert!(tally.is_empty());

    tally
        .add_words("hello", Case::Original, Encoding::Unicode)
        .expect("unicode segmentation should not fail");
    assert!(!tally.is_empty());
}

#[test]
fn test_len() {
    let mut tally = TallyMap::new();
    assert_eq!(tally.len(), 0);

    tally
        .add_words("hello", Case::Original, Encoding::Unicode)
        .expect("unicode segmentation should not fail");
    assert_eq!(tally.len(), 1);

    tally
        .add_words("world", Case::Original, Encoding::Unicode)
        .expect("unicode segmentation should not fail");
    assert_eq!(tally.len(), 2);

    tally
        .add_words("hello", Case::Original, Encoding::Unicode)
        .expect("unicode segmentation should not fail");
    assert_eq!(tally.len(), 2);
}

#[test]
fn test_values() {
    let tally = make_tally(&[("hello", 3), ("world", 1)]);
    let values: Vec<&usize> = tally.values().collect();
    assert_eq!(values.len(), 2);
    assert!(values.contains(&&3));
    assert!(values.contains(&&1));
}

#[test]
fn test_retain() {
    let mut tally = make_tally(&[("a", 1), ("ab", 2), ("abc", 3), ("abcd", 4)]);
    tally.retain(|word, _| word.len() >= 3);
    assert_eq!(tally.len(), 2);

    let words: Vec<String> = tally.into_iter().map(|(w, _)| w.into()).collect();
    assert!(words.contains(&"abc".to_string()));
    assert!(words.contains(&"abcd".to_string()));
}

#[test]
fn test_into_tally() {
    let tally = make_tally(&[("hello", 3), ("world", 1)]);
    let result: Box<[(Box<str>, usize)]> = tally.into_iter().collect();
    assert_eq!(result.len(), 2);

    let has_hello = result.iter().any(|(w, c)| w.as_ref() == "hello" && *c == 3);
    let has_world = result.iter().any(|(w, c)| w.as_ref() == "world" && *c == 1);
    assert!(has_hello);
    assert!(has_world);
}

#[test]
fn test_add_words_original_case() {
    let mut tally = TallyMap::new();
    tally
        .add_words("Hello WORLD hello", Case::Original, Encoding::Unicode)
        .expect("unicode segmentation should not fail");
    assert_eq!(tally.len(), 3);

    let words: Vec<String> = tally.into_iter().map(|(w, _)| w.into()).collect();
    assert!(words.contains(&"Hello".to_string()));
    assert!(words.contains(&"WORLD".to_string()));
    assert!(words.contains(&"hello".to_string()));
}

#[test]
fn test_add_words_lower_case() {
    let mut tally = TallyMap::new();
    tally
        .add_words("Hello WORLD hello", Case::Lower, Encoding::Unicode)
        .expect("unicode segmentation should not fail");
    assert_eq!(tally.len(), 2);

    let result: Vec<(String, usize)> = tally.into_iter().map(|(w, c)| (w.into(), c)).collect();

    let hello_count = result
        .iter()
        .find(|(w, _)| w == "hello")
        .map_or(0, |(_, c)| *c);
    assert_eq!(hello_count, 2);

    let world_count = result
        .iter()
        .find(|(w, _)| w == "world")
        .map_or(0, |(_, c)| *c);
    assert_eq!(world_count, 1);
}

#[test]
fn test_add_words_upper_case() {
    let mut tally = TallyMap::new();
    tally
        .add_words("Hello WORLD hello", Case::Upper, Encoding::Unicode)
        .expect("unicode segmentation should not fail");
    assert_eq!(tally.len(), 2);

    let result: Vec<(String, usize)> = tally.into_iter().map(|(w, c)| (w.into(), c)).collect();

    let hello_count = result
        .iter()
        .find(|(w, _)| w == "HELLO")
        .map_or(0, |(_, c)| *c);
    assert_eq!(hello_count, 2);

    let world_count = result
        .iter()
        .find(|(w, _)| w == "WORLD")
        .map_or(0, |(_, c)| *c);
    assert_eq!(world_count, 1);
}

#[test]
fn test_into_iterator() {
    let tally = make_tally(&[("hello", 3), ("world", 1)]);
    let mut count = 0;
    for (word, cnt) in tally {
        count += 1;
        assert!(word.as_ref() == "hello" || word.as_ref() == "world");
        assert!(cnt == 1 || cnt == 3);
    }
    assert_eq!(count, 2);
}

#[test]
fn test_from_iterator() {
    let items: Vec<(Word, Count)> = vec![("hello".into(), 3), ("world".into(), 1)];

    let tally: TallyMap = items.into_iter().collect();
    assert_eq!(tally.len(), 2);

    let mut result: Vec<(String, usize)> = tally.into_iter().map(|(w, c)| (w.into(), c)).collect();
    result.sort_by_key(|(w, _)| w.clone());
    assert_eq!(
        result,
        vec![("hello".to_string(), 3), ("world".to_string(), 1)]
    );
}

#[test]
fn test_from_mapped_with_parallel_bytes() {
    let content = b"I celebrate myself and sing myself";
    let view = Mapped::from(&content[..]);
    let options = Options::default().with_io(Io::ParallelBytes);

    let result = TallyMap::from_mapped(&view, &options);
    assert!(result.is_ok());

    let tally = result.expect("process test data should succeed");
    assert_eq!(tally.len(), 5);
}

#[test]
fn test_from_mapped_with_parallel_in_memory() {
    let content = b"I celebrate myself and sing myself";
    let view = Mapped::from(&content[..]);
    let options = Options::default().with_io(Io::ParallelInMemory);

    let result = TallyMap::from_mapped(&view, &options);
    assert!(result.is_ok());

    let tally = result.expect("process test data should succeed");
    assert_eq!(tally.len(), 5);
}

#[test]
fn test_default() {
    let tally = TallyMap::default();
    assert!(tally.is_empty());
    assert_eq!(tally.len(), 0);
}

#[test]
fn test_clone() {
    let tally1 = make_tally(&[("hello", 3), ("world", 1)]);
    // Test requires clone to verify original isn't consumed
    #[allow(clippy::redundant_clone)]
    let tally2 = tally1.clone();

    assert_eq!(tally1.len(), tally2.len());
    assert_eq!(tally1.len(), 2);
}

#[test]
fn test_debug() {
    let tally = make_tally(&[("hello", 3)]);
    let debug_str = format!("{tally:?}");
    assert!(debug_str.contains("TallyMap"));
    assert!(debug_str.contains("inner"));
}

#[test]
fn test_partial_eq() {
    let tally1 = make_tally(&[("hello", 3), ("world", 1)]);
    let tally2 = make_tally(&[("hello", 3), ("world", 1)]);
    let tally3 = make_tally(&[("hello", 2), ("world", 1)]);

    assert_eq!(tally1, tally2);
    assert_ne!(tally1, tally3);
}

#[test]
fn test_deref_contains_key() {
    let mut tally = TallyMap::new();
    tally
        .add_words("rust", Case::default(), Encoding::default())
        .expect("should add words");

    assert!(tally.contains_key("rust"));
    assert!(!tally.contains_key("python"));
}

#[test]
fn test_deref_get() {
    let mut tally = TallyMap::new();
    tally
        .add_words("hello world hello", Case::default(), Encoding::default())
        .expect("add test words");

    assert_eq!(tally.get("hello"), Some(&2));
    assert_eq!(tally.get("world"), Some(&1));
    assert_eq!(tally.get("missing"), None);
}

#[test]
fn test_deref_keys() {
    let mut tally = TallyMap::new();
    tally
        .add_words("apple banana", Case::default(), Encoding::default())
        .expect("should add words");

    let mut keys: Vec<_> = tally.keys().map(AsRef::as_ref).collect();
    keys.sort_unstable();
    assert_eq!(keys, vec!["apple", "banana"]);
}

#[test]
fn test_deref_iter() {
    let mut tally = TallyMap::new();
    tally
        .add_words("x y y", Case::default(), Encoding::default())
        .expect("should add words");

    let mut entries: Vec<_> = tally
        .iter()
        .map(|(word, &count)| (word.as_ref(), count))
        .collect();
    entries.sort_by_key(|(word, _)| *word);

    assert_eq!(entries, vec![("x", 1), ("y", 2)]);
}

#[test]
fn test_deref_capacity() {
    let tally = TallyMap::with_capacity(100);
    assert!(tally.capacity() >= 100);
}

#[test]
fn test_deref_clear() {
    let mut tally = make_tally(&[("hello", 3), ("world", 1)]);

    assert_eq!(tally.len(), 2);
    tally.clear();
    assert_eq!(tally.len(), 0);
    assert!(tally.is_empty());
}

#[test]
fn test_deref_shrink_to_fit() {
    let mut tally = TallyMap::with_capacity(100);
    tally
        .add_words("only_one", Case::default(), Encoding::default())
        .expect("should add words");

    let initial_capacity = tally.capacity();
    assert!(initial_capacity >= 100);

    tally.shrink_to_fit();
    assert!(tally.capacity() < initial_capacity);
}

#[test]
fn test_deref_entry() {
    let mut tally = TallyMap::new();

    // Test entry API is accessible through Deref
    tally.entry("new".into()).or_insert(5);
    assert_eq!(tally.get("new"), Some(&5));

    tally.entry("new".into()).and_modify(|count| *count += 1);
    assert_eq!(tally.get("new"), Some(&6));
}
