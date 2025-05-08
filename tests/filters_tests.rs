use word_tally::Filters;

#[test]
fn test_filters_new() {
    let min_chars = Some(3);
    let min_count = Some(2);
    let exclude_words = Some(vec!["the".to_string(), "and".to_string()]);
    let exclude_patterns: Option<Vec<String>> = None;
    let include_patterns: Option<Vec<String>> = None;

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

    let exclude_patterns = Some(vec!["^t.*".to_string()]);
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

    let exclude_patterns = None;
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
    assert!(filters.exclude_patterns().is_none());
    assert!(filters.include_patterns().is_some());

    let exclude_patterns = Some(vec!["^t.*".to_string()]);
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

    let exclude_patterns = Some(vec!["[".to_string()]);
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
    let exclude_words = Some(vec!["the".to_string(), "and".to_string()]);
    let exclude_patterns = Some(vec![]);
    let include_patterns = Some(vec![]);

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
    let exclude_words = Some(vec!["the".to_string(), "and".to_string()]);
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
