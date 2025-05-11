use word_tally::Filters;
use word_tally::filters::ExcludeWordsList;
use word_tally::patterns::InputPatterns;

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
