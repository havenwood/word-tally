use word_tally::Filters;

#[test]
fn test_create_from_args() {
    let min_chars = Some(3);
    let min_count = Some(2);
    let exclude_words = Some(vec!["the".to_string(), "and".to_string()]);
    let exclude_patterns: Option<Vec<String>> = None;
    let include_patterns: Option<Vec<String>> = None;

    let filters = Filters::create_from_args(
        &min_chars,
        &min_count,
        exclude_words.as_ref(),
        exclude_patterns.as_ref(),
        include_patterns.as_ref(),
    )
    .unwrap();

    assert_eq!(filters.min_chars().as_ref().unwrap().to_string(), "3");
    assert_eq!(filters.min_count().as_ref().unwrap().to_string(), "2");
    assert!(filters.exclude_words().is_some());
    assert!(filters.exclude_patterns().is_none());
    assert!(filters.include_patterns().is_none());

    let exclude_patterns = Some(vec!["^t.*".to_string()]);
    let filters = Filters::create_from_args(
        &min_chars,
        &min_count,
        exclude_words.as_ref(),
        exclude_patterns.as_ref(),
        include_patterns.as_ref(),
    )
    .unwrap();

    assert_eq!(filters.min_chars().as_ref().unwrap().to_string(), "3");
    assert_eq!(filters.min_count().as_ref().unwrap().to_string(), "2");
    assert!(filters.exclude_words().is_some());
    assert!(filters.exclude_patterns().is_some());
    assert!(filters.include_patterns().is_none());

    let exclude_patterns = None;
    let include_patterns = Some(vec!["^a.*".to_string()]);
    let filters = Filters::create_from_args(
        &min_chars,
        &min_count,
        exclude_words.as_ref(),
        exclude_patterns.as_ref(),
        include_patterns.as_ref(),
    )
    .unwrap();

    assert_eq!(filters.min_chars().as_ref().unwrap().to_string(), "3");
    assert_eq!(filters.min_count().as_ref().unwrap().to_string(), "2");
    assert!(filters.exclude_words().is_some());
    assert!(filters.exclude_patterns().is_none());
    assert!(filters.include_patterns().is_some());

    let exclude_patterns = Some(vec!["^t.*".to_string()]);
    let include_patterns = Some(vec!["^a.*".to_string()]);
    let filters = Filters::create_from_args(
        &min_chars,
        &min_count,
        exclude_words.as_ref(),
        exclude_patterns.as_ref(),
        include_patterns.as_ref(),
    )
    .unwrap();

    assert_eq!(filters.min_chars().as_ref().unwrap().to_string(), "3");
    assert_eq!(filters.min_count().as_ref().unwrap().to_string(), "2");
    assert!(filters.exclude_words().is_some());
    assert!(filters.exclude_patterns().is_some());
    assert!(filters.include_patterns().is_some());

    let exclude_patterns = Some(vec!["[".to_string()]);
    let result = Filters::create_from_args(
        &min_chars,
        &min_count,
        exclude_words.as_ref(),
        exclude_patterns.as_ref(),
        include_patterns.as_ref(),
    );
    assert!(result.is_err());
}

#[test]
fn test_create_from_args_with_empty_patterns() {
    let min_chars = Some(3);
    let min_count = Some(2);
    let exclude_words = Some(vec!["the".to_string(), "and".to_string()]);
    let exclude_patterns = Some(vec![]);
    let include_patterns = Some(vec![]);

    let filters = Filters::create_from_args(
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
