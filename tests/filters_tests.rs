use indexmap::IndexMap;
use word_tally::InputPatterns;
use word_tally::options::filters::{ExcludeWords, ExcludeWordsList};
use word_tally::{Case, Count, Filters, Word};

#[test]
fn test_filters_new() {
    let min_chars = Some(3);
    let min_count = Some(2);
    let exclude_words: Option<ExcludeWordsList> = Some(vec!["the".to_string(), "and".to_string()]);
    let exclude_patterns: Option<InputPatterns> = None;
    let include_patterns: Option<InputPatterns> = None;

    let filters = Filters::new(
        &min_chars,
        &min_count,
        exclude_words.as_ref(),
        exclude_patterns.as_ref(),
        include_patterns.as_ref(),
    )
    .unwrap();

    assert_eq!(filters.min_chars().unwrap(), 3);
    assert_eq!(filters.min_count().unwrap(), 2);
    assert!(filters.exclude_words().is_some());
    assert!(filters.exclude_patterns().is_none());
    assert!(filters.include_patterns().is_none());

    let exclude_patterns: Option<InputPatterns> = Some(vec!["^t.*".to_string()]);
    let filters = Filters::new(
        &min_chars,
        &min_count,
        exclude_words.as_ref(),
        exclude_patterns.as_ref(),
        include_patterns.as_ref(),
    )
    .unwrap();

    assert_eq!(filters.min_chars().unwrap(), 3);
    assert_eq!(filters.min_count().unwrap(), 2);
    assert!(filters.exclude_words().is_some());
    assert!(filters.exclude_patterns().is_some());
    assert!(filters.include_patterns().is_none());

    let exclude_patterns: Option<InputPatterns> = None;
    let include_patterns: Option<InputPatterns> = Some(vec!["^a.*".to_string()]);
    let filters = Filters::new(
        &min_chars,
        &min_count,
        exclude_words.as_ref(),
        exclude_patterns.as_ref(),
        include_patterns.as_ref(),
    )
    .unwrap();

    assert_eq!(filters.min_chars().unwrap(), 3);
    assert_eq!(filters.min_count().unwrap(), 2);
    assert!(filters.exclude_words().is_some());
    assert!(filters.exclude_patterns().is_none());
    assert!(filters.include_patterns().is_some());

    let exclude_patterns: Option<InputPatterns> = Some(vec!["^t.*".to_string()]);
    let include_patterns = Some(vec!["^a.*".to_string()]);
    let filters = Filters::new(
        &min_chars,
        &min_count,
        exclude_words.as_ref(),
        exclude_patterns.as_ref(),
        include_patterns.as_ref(),
    )
    .unwrap();

    assert_eq!(filters.min_chars().unwrap(), 3);
    assert_eq!(filters.min_count().unwrap(), 2);
    assert!(filters.exclude_words().is_some());
    assert!(filters.exclude_patterns().is_some());
    assert!(filters.include_patterns().is_some());

    let exclude_patterns: Option<InputPatterns> = Some(vec!["[".to_string()]);
    let result = Filters::new(
        &min_chars,
        &min_count,
        exclude_words.as_ref(),
        exclude_patterns.as_ref(),
        include_patterns.as_ref(),
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
        &min_chars,
        &min_count,
        exclude_words.as_ref(),
        exclude_patterns.as_ref(),
        include_patterns.as_ref(),
    )
    .unwrap();

    assert!(filters.exclude_patterns().is_none());
    assert!(filters.include_patterns().is_none());
}

#[test]
fn test_serialization_with_patterns() {
    use indexmap::IndexMap;
    use serde_json;
    use word_tally::Case;

    let min_chars = Some(3);
    let min_count = Some(2);
    let exclude_words: Option<ExcludeWordsList> = Some(vec!["the".to_string(), "and".to_string()]);
    let exclude_patterns = Some(vec![r"\d+".to_string()]);
    let include_patterns = Some(vec![r"[a-z]+".to_string()]);

    let filters = Filters::new(
        &min_chars,
        &min_count,
        exclude_words.as_ref(),
        exclude_patterns.as_ref(),
        include_patterns.as_ref(),
    )
    .unwrap();

    let json = serde_json::to_string(&filters).unwrap();
    assert!(json.contains("excludePatterns"));
    assert!(json.contains("includePatterns"));

    let deserialized_filters: Filters = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized_filters.min_chars().unwrap(), 3);
    assert_eq!(deserialized_filters.min_count().unwrap(), 2);

    let deserialized_exclude_words = deserialized_filters.exclude_words().as_ref().unwrap();
    assert_eq!(deserialized_exclude_words.len(), 2);
    assert!(deserialized_exclude_words.contains(&"the".to_string()));
    assert!(deserialized_exclude_words.contains(&"and".to_string()));

    let deserialized_exclude_patterns = deserialized_filters.exclude_patterns().as_ref().unwrap();
    assert!(deserialized_exclude_patterns.matches("123"));
    assert!(!deserialized_exclude_patterns.matches("abc"));

    let deserialized_include_patterns = deserialized_filters.include_patterns().as_ref().unwrap();
    assert!(deserialized_include_patterns.matches("abc"));
    assert!(!deserialized_include_patterns.matches("123"));

    let mut tally_map = IndexMap::new();
    tally_map.insert("the".into(), 5);
    tally_map.insert("abc".into(), 2);
    tally_map.insert("123".into(), 3);

    deserialized_filters.apply(&mut tally_map, Case::Original);

    assert_eq!(tally_map.len(), 1);
    assert!(tally_map.contains_key("abc"));
}

#[test]
fn test_multiple_regexp_patterns() {
    use indexmap::IndexMap;
    use word_tally::Case;

    let multiple_patterns = vec![
        r"^[aeiou].*".to_string(),
        r".*ing$".to_string(),
        r"^[A-Z].*".to_string(),
    ];

    let filters = Filters::default()
        .with_include_patterns(&multiple_patterns)
        .unwrap();

    let mut tally_map = IndexMap::new();
    tally_map.insert("apple".into(), 3);
    tally_map.insert("banana".into(), 2);
    tally_map.insert("eating".into(), 5);
    tally_map.insert("orange".into(), 1);
    tally_map.insert("running".into(), 4);
    tally_map.insert("Example".into(), 2);
    tally_map.insert("test".into(), 6);

    filters.apply(&mut tally_map, Case::Original);

    assert_eq!(tally_map.len(), 5);
    assert!(tally_map.contains_key("apple"));
    assert!(!tally_map.contains_key("banana"));
    assert!(tally_map.contains_key("eating"));
    assert!(tally_map.contains_key("orange"));
    assert!(tally_map.contains_key("running"));
    assert!(tally_map.contains_key("Example"));
    assert!(!tally_map.contains_key("test"));
}

#[test]
fn test_patterns_accessors() {
    let exclude_patterns = vec![r"\d+".to_string()];
    let include_patterns = vec![r"[a-z]+".to_string()];

    let filters = Filters::default()
        .with_exclude_patterns(&exclude_patterns)
        .unwrap()
        .with_include_patterns(&include_patterns)
        .unwrap();

    let exclude = filters.exclude_patterns().as_ref().unwrap();
    let include = filters.include_patterns().as_ref().unwrap();

    assert_eq!(&exclude.as_patterns()[0], r"\d+");
    assert_eq!(&include.as_patterns()[0], r"[a-z]+");

    assert!(exclude.matches("123"));
    assert!(!exclude.matches("abc"));

    assert!(include.matches("abc"));
    assert!(!include.matches("123"));

    assert_eq!(exclude.len(), 1);
    assert!(!exclude.is_empty());

    assert_eq!(include.len(), 1);
    assert!(!include.is_empty());

    let exclude_ref: &[String] = exclude.as_ref();
    let include_ref: &[String] = include.as_ref();

    assert_eq!(&exclude_ref[0], r"\d+");
    assert_eq!(&include_ref[0], r"[a-z]+");
}

#[test]
fn test_with_unescaped_exclude_words() {
    let words = vec![
        r#"\n"#.to_string(),
        r#"\t"#.to_string(),
        r#"\\"#.to_string(),
        r#"\""#.to_string(),
        r#"hello\nworld"#.to_string(),
    ];

    let filters = Filters::default()
        .with_unescaped_exclude_words(&words)
        .unwrap();

    let exclude_words = filters.exclude_words().as_ref().unwrap();
    assert_eq!(exclude_words.len(), 5);
    assert!(exclude_words.contains(&"\n".to_string()));
    assert!(exclude_words.contains(&"\t".to_string()));
    assert!(exclude_words.contains(&"\\".to_string()));
    assert!(exclude_words.contains(&"\"".to_string()));
    assert!(exclude_words.contains(&"hello\nworld".to_string()));
}

#[test]
fn test_with_unescaped_exclude_words_error() {
    let words = vec![r#"\x"#.to_string()];

    let result = Filters::default().with_unescaped_exclude_words(&words);

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("failed to unescape"));
}

#[test]
fn test_with_unescaped_exclude_words_unicode() {
    let words = vec![
        r#"\u{1F308}"#.to_string(), // Rainbow emoji
        r#"\u{00E9}"#.to_string(),  // é (e with acute)
        r#"\u{0301}"#.to_string(),  // Combining acute accent
    ];

    let filters = Filters::default()
        .with_unescaped_exclude_words(&words)
        .unwrap();

    let exclude_words = filters.exclude_words().as_ref().unwrap();
    assert_eq!(exclude_words.len(), 3);
    assert!(exclude_words.contains(&"🌈".to_string()));
    assert!(exclude_words.contains(&"é".to_string()));
    assert!(exclude_words.contains(&"\u{0301}".to_string()));
}

#[test]
fn test_exclude_words_display() {
    let words = vec![
        "apple".to_string(),
        "banana".to_string(),
        "cherry".to_string(),
    ];
    let exclude_words = ExcludeWords(words);

    assert_eq!(exclude_words.to_string(), "apple,banana,cherry");

    let empty = ExcludeWords(vec![]);
    assert_eq!(empty.to_string(), "");

    let single = ExcludeWords(vec!["word".to_string()]);
    assert_eq!(single.to_string(), "word");
}

#[test]
fn test_exclude_words_from() {
    let words = vec!["test".to_string(), "words".to_string()];
    let exclude_words: ExcludeWords = words.clone().into();

    assert_eq!(exclude_words.0, words);
}

#[test]
fn test_exclude_words_as_ref() {
    let words = vec!["one".to_string(), "two".to_string()];
    let exclude_words = ExcludeWords(words.clone());

    let words_ref: &ExcludeWordsList = exclude_words.as_ref();
    assert_eq!(words_ref, &words);
}

#[test]
fn test_exclude_words_deref() {
    let words = vec!["alpha".to_string(), "beta".to_string()];
    let exclude_words = ExcludeWords(words.clone());

    assert_eq!(*exclude_words, words);
    assert_eq!(exclude_words.len(), 2);
    assert!(exclude_words.contains(&"alpha".to_string()));
}

#[test]
fn test_all_filters_combined() {
    let mut tally_map: IndexMap<Word, Count> = IndexMap::new();
    tally_map.insert("a".into(), 5);
    tally_map.insert("the".into(), 1);
    tally_map.insert("hello".into(), 3);
    tally_map.insert("world123".into(), 4);
    tally_map.insert("goodnight".into(), 6);
    tally_map.insert("goodbye".into(), 7);

    let filters = Filters::default()
        .with_min_chars(3)
        .with_min_count(2)
        .with_exclude_words(vec!["hello".to_string()])
        .with_exclude_patterns(&vec![r#"\d+"#.to_string()])
        .unwrap()
        .with_include_patterns(&vec![r"^good".to_string()])
        .unwrap();

    filters.apply(&mut tally_map, Case::Lower);

    assert_eq!(tally_map.len(), 2);
    assert!(tally_map.contains_key("goodbye"));
    assert!(tally_map.contains_key("goodnight"));
    assert_eq!(tally_map["goodbye"], 7);
    assert_eq!(tally_map["goodnight"], 6);
}

#[test]
fn test_overlapping_patterns() {
    let mut tally_map: IndexMap<Word, Count> = IndexMap::new();
    tally_map.insert("good".into(), 3);
    tally_map.insert("goodbye".into(), 4);
    tally_map.insert("goodness".into(), 5);
    tally_map.insert("great".into(), 6);

    let filters = Filters::default()
        .with_include_patterns(&vec![
            r"^good".to_string(),
            r".*bye$".to_string(),
            r"^g.*t$".to_string(),
        ])
        .unwrap();

    filters.apply(&mut tally_map, Case::Lower);

    assert_eq!(tally_map.len(), 4);
}

#[test]
fn test_conflicting_patterns() {
    let mut tally_map: IndexMap<Word, Count> = IndexMap::new();
    tally_map.insert("good".into(), 3);
    tally_map.insert("goodbye".into(), 4);
    tally_map.insert("bad".into(), 5);

    let filters = Filters::default()
        .with_include_patterns(&vec![r"^good".to_string()])
        .unwrap()
        .with_exclude_patterns(&vec![r"bye$".to_string()])
        .unwrap();

    filters.apply(&mut tally_map, Case::Lower);

    assert_eq!(tally_map.len(), 1);
    assert!(tally_map.contains_key("good"));
}

#[test]
fn test_case_normalization_with_exclude_words() {
    let mut tally_map: IndexMap<Word, Count> = IndexMap::new();
    tally_map.insert("hello".into(), 3);
    tally_map.insert("world".into(), 4);
    tally_map.insert("test".into(), 5);

    let filters =
        Filters::default().with_exclude_words(vec!["HELLO".to_string(), "World".to_string()]);

    filters.apply(&mut tally_map, Case::Lower);

    assert_eq!(tally_map.len(), 1);
    assert!(tally_map.contains_key("test"));
}

#[test]
fn test_empty_filters() {
    let mut tally_map: IndexMap<Word, Count> = IndexMap::new();
    tally_map.insert("hello".into(), 3);
    tally_map.insert("world".into(), 4);

    let original_len = tally_map.len();

    let filters = Filters::default();
    filters.apply(&mut tally_map, Case::Lower);

    assert_eq!(tally_map.len(), original_len);
}

#[test]
fn test_unicode_grapheme_counting() {
    let mut tally_map: IndexMap<Word, Count> = IndexMap::new();

    tally_map.insert("é".into(), 1);
    tally_map.insert("e\u{0301}".into(), 1);
    tally_map.insert("🇺🇸".into(), 1);
    tally_map.insert("👍🏻".into(), 1);
    tally_map.insert("नमस्ते".into(), 1);
    tally_map.insert("🧑‍🦽".into(), 1);
    tally_map.insert("a".into(), 1);

    let filters = Filters::default().with_min_chars(2);
    filters.apply(&mut tally_map, Case::Lower);

    assert!(!tally_map.contains_key("é"));
    assert!(!tally_map.contains_key("e\u{0301}"));
    assert!(!tally_map.contains_key("🇺🇸"));
    assert!(!tally_map.contains_key("👍🏻"));
    assert!(tally_map.contains_key("नमस्ते"));
    assert!(!tally_map.contains_key("🧑‍🦽"));
    assert!(!tally_map.contains_key("a"));
}

#[test]
fn test_complex_unicode_patterns() {
    let mut tally_map: IndexMap<Word, Count> = IndexMap::new();
    tally_map.insert("café".into(), 1);
    tally_map.insert("naïve".into(), 1);
    tally_map.insert("piñata".into(), 1);
    tally_map.insert("hello".into(), 1);

    let filters = Filters::default()
        .with_include_patterns(&vec![r"[àáâãäåèéêëìíîïòóôõöùúûüýÿñç]".to_string()])
        .unwrap();

    filters.apply(&mut tally_map, Case::Lower);

    assert_eq!(tally_map.len(), 3);
    assert!(!tally_map.contains_key("hello"));
}

#[test]
fn test_filter_ordering() {
    let mut tally_map1: IndexMap<Word, Count> = IndexMap::new();
    tally_map1.insert("test".into(), 1);
    tally_map1.insert("testing".into(), 10);
    tally_map1.insert("tested".into(), 5);

    let mut tally_map2 = tally_map1.clone();

    let filters1 = Filters::default().with_min_count(2).with_min_chars(6);

    let filters2 = Filters::default().with_min_chars(6).with_min_count(2);

    filters1.apply(&mut tally_map1, Case::Lower);
    filters2.apply(&mut tally_map2, Case::Lower);

    assert_eq!(tally_map1, tally_map2);
    assert_eq!(tally_map1.len(), 2);
    assert!(tally_map1.contains_key("testing"));
    assert!(tally_map1.contains_key("tested"));
}
