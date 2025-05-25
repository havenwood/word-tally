use word_tally::InputPatterns;
use word_tally::options::filters::{ExcludeWords, ExcludeWordsList};
use word_tally::{Case, Filters, TallyMap};

// Helper function to create TallyMap from counts
fn tally_map_from_counts(counts: &[(&str, usize)]) -> TallyMap {
    let mut tally = TallyMap::new();
    for (word, count) in counts {
        for _ in 0..*count {
            tally.extend_from_str(word, Case::Original);
        }
    }
    tally
}

// Helper function to check if a word exists in the TallyMap
fn has_word(tally: &TallyMap, word: &str) -> bool {
    tally.clone().into_iter().any(|(w, _)| w.as_ref() == word)
}

#[test]
fn test_filters_new() {
    let min_chars = Some(3);
    let min_count = Some(2);
    let exclude_words: Option<ExcludeWordsList> = Some(vec!["the".to_string(), "and".to_string()]);
    let exclude_patterns: Option<InputPatterns> = None;
    let include_patterns: Option<InputPatterns> = None;

    let filters = Filters::new(
        min_chars,
        min_count,
        exclude_words.clone(),
        exclude_patterns,
        include_patterns.clone(),
    )
    .expect("execute operation");

    assert_eq!(filters.min_chars().expect("process test"), 3);
    assert_eq!(filters.min_count().expect("process test"), 2);
    assert!(filters.exclude_words().is_some());
    assert!(filters.exclude_patterns().is_none());
    assert!(filters.include_patterns().is_none());

    let exclude_patterns: Option<InputPatterns> = Some(vec!["^t.*".to_string()]);
    let filters = Filters::new(
        min_chars,
        min_count,
        exclude_words.clone(),
        exclude_patterns,
        include_patterns,
    )
    .expect("execute operation");

    assert_eq!(filters.min_chars().expect("process test"), 3);
    assert_eq!(filters.min_count().expect("process test"), 2);
    assert!(filters.exclude_words().is_some());
    assert!(filters.exclude_patterns().is_some());
    assert!(filters.include_patterns().is_none());

    let exclude_patterns: Option<InputPatterns> = None;
    let include_patterns: Option<InputPatterns> = Some(vec!["^a.*".to_string()]);
    let filters = Filters::new(
        min_chars,
        min_count,
        exclude_words.clone(),
        exclude_patterns,
        include_patterns,
    )
    .expect("execute operation");

    assert_eq!(filters.min_chars().expect("process test"), 3);
    assert_eq!(filters.min_count().expect("process test"), 2);
    assert!(filters.exclude_words().is_some());
    assert!(filters.exclude_patterns().is_none());
    assert!(filters.include_patterns().is_some());

    let exclude_patterns: Option<InputPatterns> = Some(vec!["^t.*".to_string()]);
    let include_patterns = Some(vec!["^a.*".to_string()]);
    let filters = Filters::new(
        min_chars,
        min_count,
        exclude_words.clone(),
        exclude_patterns,
        include_patterns.clone(),
    )
    .expect("execute operation");

    assert_eq!(filters.min_chars().expect("process test"), 3);
    assert_eq!(filters.min_count().expect("process test"), 2);
    assert!(filters.exclude_words().is_some());
    assert!(filters.exclude_patterns().is_some());
    assert!(filters.include_patterns().is_some());

    let exclude_patterns: Option<InputPatterns> = Some(vec!["[".to_string()]);
    let result = Filters::new(
        min_chars,
        min_count,
        exclude_words,
        exclude_patterns,
        include_patterns,
    );
    assert!(result.is_err());
}

#[test]
fn test_filters_with_empty_patterns() {
    let min_chars = Some(3);
    let min_count = Some(2);
    let exclude_words: Option<ExcludeWordsList> = Some(vec!["the".to_string(), "and".to_string()]);
    let exclude_patterns: Option<InputPatterns> = Some(vec![]);
    let include_patterns: Option<InputPatterns> = Some(vec![]);

    let filters = Filters::new(
        min_chars,
        min_count,
        exclude_words,
        exclude_patterns,
        include_patterns,
    )
    .expect("execute operation");

    assert_eq!(filters.min_chars().expect("process test"), 3);
    assert_eq!(filters.min_count().expect("process test"), 2);
    assert!(filters.exclude_words().is_some());
    assert!(filters.exclude_patterns().is_none());
    assert!(filters.include_patterns().is_none());
}

#[test]
fn test_serialization_with_patterns() {
    use serde_json;

    let min_chars = Some(3);
    let min_count = Some(2);
    let exclude_words: Option<ExcludeWordsList> = Some(vec!["the".to_string(), "and".to_string()]);
    let exclude_patterns = Some(vec![r"\d+".to_string()]);
    let include_patterns = Some(vec![r"[a-z]+".to_string()]);

    let filters = Filters::new(
        min_chars,
        min_count,
        exclude_words,
        exclude_patterns,
        include_patterns,
    )
    .expect("execute operation");

    let serialized = serde_json::to_string(&filters).expect("serialize JSON");
    let deserialized: Filters = serde_json::from_str(&serialized).expect("deserialize JSON");

    assert_eq!(deserialized.min_chars(), filters.min_chars());
    assert_eq!(deserialized.min_count(), filters.min_count());

    let serialized = r#"{
    "excludePatterns": ["the", "is"],
    "includePatterns": ["test.*"]
  }"#;

    let deserialized_filters: Filters = serde_json::from_str(serialized).expect("deserialize JSON");
    let deserialized_exclude_patterns = deserialized_filters
        .exclude_patterns()
        .expect("process test");
    assert!(deserialized_exclude_patterns.matches("the"));
    assert!(deserialized_exclude_patterns.matches("is"));

    let deserialized_include_patterns = deserialized_filters
        .include_patterns()
        .expect("process test");
    assert!(deserialized_include_patterns.matches("test"));
    assert!(!deserialized_include_patterns.matches("123"));

    let mut tally_map = TallyMap::new();
    tally_map.extend_from_str("the the the the the test test 123 123 123", Case::Original);

    deserialized_filters.apply(&mut tally_map, Case::Original);

    assert_eq!(tally_map.len(), 1);
    assert!(has_word(&tally_map, "test"));
}

#[test]
fn test_multiple_regexp_patterns() {
    let multiple_patterns = vec![
        r"^[aeiou].*".to_string(),
        r".*ing$".to_string(),
        r"^Example$".to_string(),
    ];

    let filters = Filters::default()
        .with_include_patterns(&multiple_patterns)
        .expect("execute operation");

    let mut tally_map = TallyMap::new();
    tally_map.extend_from_str("apple apple apple banana banana eating eating eating eating eating orange running running running running Example Example test test test test test test", Case::Original);

    filters.apply(&mut tally_map, Case::Original);

    assert_eq!(tally_map.len(), 5);
    assert!(has_word(&tally_map, "apple"));
    assert!(!has_word(&tally_map, "banana"));
    assert!(has_word(&tally_map, "eating"));
    assert!(has_word(&tally_map, "orange"));
    assert!(has_word(&tally_map, "running"));
    assert!(has_word(&tally_map, "Example"));
    assert!(!has_word(&tally_map, "test"));
}

#[test]
fn test_patterns_accessors() {
    let exclude_patterns = vec![r"bad".to_string()];
    let include_patterns = vec![r"good".to_string()];

    let filters = Filters::default()
        .with_exclude_patterns(&exclude_patterns)
        .expect("execute operation")
        .with_include_patterns(&include_patterns)
        .expect("execute operation");

    let excl_patterns = filters.exclude_patterns().expect("process test");
    assert!(excl_patterns.matches("bad"));
    assert!(!excl_patterns.matches("good"));

    let incl_patterns = filters.include_patterns().expect("process test");
    assert!(incl_patterns.matches("good"));
    assert!(!incl_patterns.matches("bad"));
}

#[test]
fn test_builder() {
    let min_chars = Some(4);
    let min_count = Some(3);
    let exclude_words: ExcludeWordsList = vec!["stop".to_string(), "the".to_string()];

    let filters = Filters::default()
        .with_min_chars(4)
        .with_min_count(3)
        .with_exclude_words(exclude_words);

    assert_eq!(filters.min_chars(), min_chars);
    assert_eq!(filters.min_count(), min_count);
}

#[test]
fn test_default() {
    let filters = Filters::default();

    assert_eq!(filters.min_chars(), None);
    assert_eq!(filters.min_count(), None);
    assert_eq!(filters.exclude_words(), None);
}

#[test]
fn test_pattern_serialization() {
    use serde_json;

    let multi_patterns = vec![
        r"^test.*".to_string(),
        r".*ing$".to_string(),
        r"[0-9]+".to_string(),
    ];
    let filters = Filters::default()
        .with_include_patterns(&multi_patterns)
        .expect("execute operation");

    let serialized = serde_json::to_string(&filters).expect("serialize JSON");
    let deserialized: Filters = serde_json::from_str(&serialized).expect("deserialize JSON");

    let patterns = deserialized.include_patterns().expect("process test");
    assert!(patterns.matches("testing"));
    assert!(patterns.matches("running"));
    assert!(patterns.matches("123"));
    assert!(!patterns.matches("hello"));
}

#[test]
fn test_filter_combination() {
    let exclude_words = vec!["the".to_string(), "and".to_string()];
    let exclude_patterns = vec![r"\d+".to_string()];
    let include_patterns = vec![r"^test.*".to_string()];

    let filters = Filters::default()
        .with_min_chars(3)
        .with_min_count(2)
        .with_exclude_words(exclude_words)
        .with_exclude_patterns(&exclude_patterns)
        .expect("execute operation")
        .with_include_patterns(&include_patterns)
        .expect("execute operation");

    let mut tally_map = tally_map_from_counts(&[
        ("test", 3),
        ("testing", 2),
        ("the", 5),
        ("and", 4),
        ("test123", 2),
        ("te", 3),
        ("testcase", 1),
    ]);

    filters.apply(&mut tally_map, Case::Lower);

    assert_eq!(tally_map.len(), 2);
    assert!(has_word(&tally_map, "test"));
    assert!(has_word(&tally_map, "testing"));
}

#[test]
fn test_invalid_include_patterns() {
    use word_tally::InputPatterns;

    let invalid_pattern: InputPatterns = vec![r"[".to_string()];
    let result = Filters::default().with_include_patterns(&invalid_pattern);
    assert!(result.is_err());
}

#[test]
fn test_case_sensitive_exclude_patterns() {
    let exclude_patterns = vec![r"Test".to_string()];

    let filters = Filters::default()
        .with_exclude_patterns(&exclude_patterns)
        .expect("execute operation");

    let mut tally_map = tally_map_from_counts(&[("Test", 3), ("test", 2), ("TEST", 1)]);

    filters.apply(&mut tally_map, Case::Original);

    assert_eq!(tally_map.len(), 2);
    assert!(!has_word(&tally_map, "Test"));
    assert!(has_word(&tally_map, "test"));
    assert!(has_word(&tally_map, "TEST"));
}

#[test]
fn test_min_chars_filter() {
    let mut tally_map = tally_map_from_counts(&[("a", 5), ("ab", 4), ("abc", 3), ("abcd", 2)]);

    let filters = Filters::default().with_min_chars(3);
    filters.apply(&mut tally_map, Case::Original);

    assert_eq!(tally_map.len(), 2);
    assert!(has_word(&tally_map, "abc"));
    assert!(has_word(&tally_map, "abcd"));
    assert!(!has_word(&tally_map, "a"));
    assert!(!has_word(&tally_map, "ab"));
}

#[test]
fn test_min_count_filter() {
    let mut tally_map =
        tally_map_from_counts(&[("hello", 5), ("world", 3), ("test", 2), ("example", 4)]);

    let filters = Filters::default().with_min_count(3);
    filters.apply(&mut tally_map, Case::Original);

    assert_eq!(tally_map.len(), 3);
    assert!(has_word(&tally_map, "hello"));
    assert!(has_word(&tally_map, "world"));
    assert!(!has_word(&tally_map, "test"));
    assert!(has_word(&tally_map, "example"));
}

#[test]
fn test_exclude_words_filter() {
    let mut tally_map =
        tally_map_from_counts(&[("the", 10), ("quick", 5), ("brown", 3), ("fox", 7)]);

    let exclude_words = vec!["the".to_string(), "fox".to_string()];
    let filters = Filters::default().with_exclude_words(exclude_words);
    filters.apply(&mut tally_map, Case::Lower);

    assert_eq!(tally_map.len(), 2);
    assert!(!has_word(&tally_map, "the"));
    assert!(has_word(&tally_map, "quick"));
    assert!(has_word(&tally_map, "brown"));
    assert!(!has_word(&tally_map, "fox"));
}

#[test]
fn test_overlapping_patterns() {
    let mut tally_map =
        tally_map_from_counts(&[("good", 3), ("goodbye", 4), ("goodness", 5), ("great", 6)]);

    let filters = Filters::default()
        .with_include_patterns(&vec![r"^good".to_string(), r"^g.*t$".to_string()])
        .expect("execute operation");

    filters.apply(&mut tally_map, Case::Lower);

    assert_eq!(tally_map.len(), 4);
}

#[test]
fn test_include_exclude_patterns_combination() {
    let mut tally_map = tally_map_from_counts(&[("good", 3), ("goodbye", 4), ("bad", 5)]);

    let filters = Filters::default()
        .with_include_patterns(&vec![r"^good".to_string()])
        .expect("execute operation")
        .with_exclude_patterns(&vec![r"bye$".to_string()])
        .expect("execute operation");

    filters.apply(&mut tally_map, Case::Lower);

    assert_eq!(tally_map.len(), 1);
    assert!(has_word(&tally_map, "good"));
}

#[test]
fn test_case_normalization_with_exclude_words() {
    // Create a TallyMap with case-normalized words
    let mut tally_map = TallyMap::new();
    tally_map.extend_from_str(
        "hello hello hello Hello Hello HELLO world world world world",
        Case::Lower,
    );

    let exclude_words = vec!["hello".to_string()];
    let filters = Filters::default().with_exclude_words(exclude_words);
    filters.apply(&mut tally_map, Case::Lower);

    assert_eq!(tally_map.len(), 1);
    assert!(has_word(&tally_map, "world"));
}

#[test]
fn test_exclude_empty_patterns() {
    let empty_patterns: Vec<String> = vec![];
    let filters = Filters::default()
        .with_exclude_patterns(&empty_patterns)
        .expect("execute operation");

    assert!(filters.exclude_patterns().is_none());
}

#[test]
fn test_include_empty_patterns() {
    let empty_patterns: Vec<String> = vec![];
    let filters = Filters::default()
        .with_include_patterns(&empty_patterns)
        .expect("execute operation");

    assert!(filters.include_patterns().is_none());
}

#[test]
fn test_pattern_precedence() {
    let mut tally_map =
        tally_map_from_counts(&[("test", 3), ("testing", 2), ("tester", 4), ("rest", 1)]);

    let filters = Filters::default()
        .with_include_patterns(&vec![r".*est.*".to_string()])
        .expect("execute operation")
        .with_exclude_patterns(&vec![r"^test$".to_string()])
        .expect("execute operation");

    filters.apply(&mut tally_map, Case::Lower);

    assert_eq!(tally_map.len(), 3);
    assert!(!has_word(&tally_map, "test"));
    assert!(has_word(&tally_map, "testing"));
    assert!(has_word(&tally_map, "tester"));
    assert!(has_word(&tally_map, "rest"));
}

#[test]
fn test_unicode_min_chars() {
    let mut tally_map = tally_map_from_counts(&[("a", 1), ("café", 1), ("naive", 1), ("naïve", 1)]);

    let filters = Filters::default().with_min_chars(4);
    filters.apply(&mut tally_map, Case::Lower);

    assert_eq!(tally_map.len(), 3);
    assert!(!has_word(&tally_map, "a"));
    assert!(has_word(&tally_map, "café"));
    assert!(has_word(&tally_map, "naive"));
    assert!(has_word(&tally_map, "naïve"));
}

#[test]
fn test_complex_filter_combination() {
    let mut tally_map1 = tally_map_from_counts(&[
        ("hello", 5),
        ("world", 3),
        ("test", 2),
        ("testing", 4),
        ("the", 10),
        ("a", 8),
        ("12345", 2),
    ]);

    let mut tally_map2 = tally_map1.clone();

    let filters1 = Filters::default()
        .with_min_chars(2)
        .with_min_count(3)
        .with_exclude_words(vec!["the".to_string()])
        .with_exclude_patterns(&vec![r"\d+".to_string()])
        .expect("execute operation");

    let filters2 = Filters::default()
        .with_min_chars(2)
        .with_min_count(3)
        .with_exclude_words(vec!["the".to_string()])
        .with_exclude_patterns(&vec![r"\d+".to_string()])
        .expect("execute operation");

    filters1.apply(&mut tally_map1, Case::Lower);
    filters2.apply(&mut tally_map2, Case::Lower);

    assert_eq!(tally_map1.len(), 3);
    assert_eq!(tally_map2.len(), 3);
}

#[test]
fn test_with_unescaped_exclude_words() {
    let escaped_words = vec!["\\n".to_string(), "\\t".to_string(), "\\r".to_string()];

    let filters = Filters::default()
        .with_unescaped_exclude_words(&escaped_words)
        .expect("execute operation");

    let exclude_words = filters.exclude_words().expect("process test");
    let words_list = exclude_words.as_ref();

    assert_eq!(words_list.len(), 3);
    assert_eq!(words_list[0], "\n");
    assert_eq!(words_list[1], "\t");
    assert_eq!(words_list[2], "\r");
}

#[test]
fn test_display_exclude_words() {
    let words = vec!["hello".to_string(), "world".to_string()];
    let exclude_words = ExcludeWords(words);
    assert_eq!(format!("{exclude_words}"), "hello,world");
}

#[test]
fn test_filters_display() {
    let filters = Filters::default().with_min_chars(3).with_min_count(2);

    assert!(format!("{filters:?}").contains("min_chars: Some(3)"));
    assert!(format!("{filters:?}").contains("min_count: Some(2)"));
}
