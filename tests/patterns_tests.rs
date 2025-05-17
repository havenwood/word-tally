use word_tally::{ExcludePatterns, IncludePatterns};

#[test]
fn test_exclude_patterns_new() {
    let patterns = ExcludePatterns::new(vec!["test".to_string()]).unwrap();
    assert_eq!(patterns.len(), 1);
    assert!(!patterns.is_empty());
}

#[test]
fn test_exclude_patterns_empty() {
    let patterns = ExcludePatterns::default();
    assert_eq!(patterns.len(), 0);
    assert!(patterns.is_empty());
}

#[test]
fn test_exclude_patterns_matches() {
    let patterns = ExcludePatterns::new(vec!["^test".to_string(), "ing$".to_string()]).unwrap();

    assert!(patterns.matches("testing"));
    assert!(patterns.matches("test"));
    assert!(patterns.matches("running"));
    assert!(!patterns.matches("run"));
}

#[test]
fn test_exclude_patterns_as_patterns() {
    let input = vec!["^pre".to_string(), "^un".to_string()];
    let patterns = ExcludePatterns::new(input.clone()).unwrap();
    assert_eq!(patterns.as_patterns(), input.as_slice());
}

#[test]
fn test_exclude_patterns_try_from() {
    let input = vec!["pattern1".to_string(), "pattern2".to_string()];
    let patterns = ExcludePatterns::try_from(input.as_slice()).unwrap();
    assert_eq!(patterns.len(), 2);
}

#[test]
fn test_exclude_patterns_invalid_regex() {
    let result = ExcludePatterns::new(vec!["[invalid".to_string()]);
    assert!(result.is_err());
}

#[test]
fn test_include_patterns_new() {
    let patterns = IncludePatterns::new(vec!["test".to_string()]).unwrap();
    assert_eq!(patterns.len(), 1);
    assert!(!patterns.is_empty());
}

#[test]
fn test_include_patterns_empty() {
    let patterns = IncludePatterns::default();
    assert_eq!(patterns.len(), 0);
    assert!(patterns.is_empty());
}

#[test]
fn test_include_patterns_matches() {
    let patterns = IncludePatterns::new(vec!["[aeiou]".to_string()]).unwrap();

    assert!(patterns.matches("test"));
    assert!(patterns.matches("hello"));
    assert!(!patterns.matches("rhythm"));
    assert!(!patterns.matches("xyz"));
}

#[test]
fn test_include_patterns_multiple() {
    let patterns = IncludePatterns::new(vec!["^pre".to_string(), "^un".to_string()]).unwrap();

    assert!(patterns.matches("prevent"));
    assert!(patterns.matches("unlike"));
    assert!(!patterns.matches("likely"));
    assert!(!patterns.matches("post"));
}

#[test]
fn test_include_patterns_as_patterns() {
    let input = vec!["test1".to_string(), "test2".to_string()];
    let patterns = IncludePatterns::new(input.clone()).unwrap();
    assert_eq!(patterns.as_patterns(), input.as_slice());
}

#[test]
fn test_include_patterns_try_from() {
    let input = vec!["^start".to_string(), "end$".to_string()];
    let patterns = IncludePatterns::try_from(input.as_slice()).unwrap();
    assert_eq!(patterns.len(), 2);
}

#[test]
fn test_include_patterns_invalid_regex() {
    let result = IncludePatterns::new(vec!["[invalid".to_string()]);
    assert!(result.is_err());
}

#[test]
fn test_complex_regex_patterns() {
    let patterns = ExcludePatterns::new(vec![r"\btest\b".to_string()]).unwrap();
    assert!(patterns.matches("test"));
    assert!(!patterns.matches("testing"));

    let patterns = IncludePatterns::new(vec![r"[0-9]+".to_string()]).unwrap();
    assert!(patterns.matches("abc123"));
    assert!(!patterns.matches("abc"));

    let patterns = ExcludePatterns::new(vec![r"^the$".to_string()]).unwrap();
    assert!(patterns.matches("the"));
    assert!(!patterns.matches("them"));
    assert!(!patterns.matches("soothe"));
}

#[test]
fn test_exclude_patterns_as_ref() {
    let input = vec!["pattern".to_string()];
    let patterns = ExcludePatterns::new(input.clone()).unwrap();
    let as_ref: &[String] = patterns.as_ref();
    assert_eq!(as_ref, input.as_slice());
}

#[test]
fn test_include_patterns_as_ref() {
    let input = vec!["pattern".to_string()];
    let patterns = IncludePatterns::new(input.clone()).unwrap();
    let as_ref: &[String] = patterns.as_ref();
    assert_eq!(as_ref, input.as_slice());
}

#[test]
fn test_exclude_patterns_eq() {
    let patterns1 = ExcludePatterns::new(vec!["test".to_string()]).unwrap();
    let patterns2 = ExcludePatterns::new(vec!["test".to_string()]).unwrap();
    let patterns3 = ExcludePatterns::new(vec!["other".to_string()]).unwrap();

    assert_eq!(patterns1, patterns2);
    assert_ne!(patterns1, patterns3);
}

#[test]
fn test_include_patterns_eq() {
    let patterns1 = IncludePatterns::new(vec!["test".to_string()]).unwrap();
    let patterns2 = IncludePatterns::new(vec!["test".to_string()]).unwrap();
    let patterns3 = IncludePatterns::new(vec!["other".to_string()]).unwrap();

    assert_eq!(patterns1, patterns2);
    assert_ne!(patterns1, patterns3);
}

#[test]
fn test_exclude_patterns_ord() {
    let patterns1 = ExcludePatterns::new(vec!["aaa".to_string()]).unwrap();
    let patterns2 = ExcludePatterns::new(vec!["bbb".to_string()]).unwrap();

    assert!(patterns1 < patterns2);
    assert!(patterns2 > patterns1);
}

#[test]
fn test_include_patterns_ord() {
    let patterns1 = IncludePatterns::new(vec!["aaa".to_string()]).unwrap();
    let patterns2 = IncludePatterns::new(vec!["bbb".to_string()]).unwrap();

    assert!(patterns1 < patterns2);
    assert!(patterns2 > patterns1);
}

#[test]
// Testing hash implementation with interior mutability is safe here
#[allow(clippy::mutable_key_type)]
fn test_exclude_patterns_hash() {
    use std::collections::HashSet;

    let patterns1 = ExcludePatterns::new(vec!["test".to_string()]).unwrap();
    let patterns2 = ExcludePatterns::new(vec!["test".to_string()]).unwrap();

    let mut set = HashSet::new();
    set.insert(patterns1);
    assert!(set.contains(&patterns2));
}

#[test]
// Testing hash implementation with interior mutability is safe here
#[allow(clippy::mutable_key_type)]
fn test_include_patterns_hash() {
    use std::collections::HashSet;

    let patterns1 = IncludePatterns::new(vec!["test".to_string()]).unwrap();
    let patterns2 = IncludePatterns::new(vec!["test".to_string()]).unwrap();

    let mut set = HashSet::new();
    set.insert(patterns1);
    assert!(set.contains(&patterns2));
}

#[test]
fn test_exclude_patterns_display() {
    let patterns =
        ExcludePatterns::new(vec!["pattern1".to_string(), "pattern2".to_string()]).unwrap();
    assert_eq!(patterns.to_string(), "pattern1,pattern2");
}

#[test]
fn test_include_patterns_display() {
    let patterns =
        IncludePatterns::new(vec!["pattern1".to_string(), "pattern2".to_string()]).unwrap();
    assert_eq!(patterns.to_string(), "pattern1,pattern2");
}

#[test]
fn test_exclude_patterns_serde() {
    let patterns = ExcludePatterns::new(vec!["test1".to_string(), "test2".to_string()]).unwrap();

    let serialized = serde_json::to_string(&patterns).unwrap();
    assert_eq!(serialized, r#"["test1","test2"]"#);

    let deserialized: ExcludePatterns = serde_json::from_str(&serialized).unwrap();
    assert_eq!(patterns, deserialized);
}

#[test]
fn test_include_patterns_serde() {
    let patterns = IncludePatterns::new(vec!["test1".to_string(), "test2".to_string()]).unwrap();

    let serialized = serde_json::to_string(&patterns).unwrap();
    assert_eq!(serialized, r#"["test1","test2"]"#);

    let deserialized: IncludePatterns = serde_json::from_str(&serialized).unwrap();
    assert_eq!(patterns, deserialized);
}

#[test]
fn test_exclude_patterns_deserialize_error() {
    let invalid = r#"["[invalid"]"#;
    let result: Result<ExcludePatterns, _> = serde_json::from_str(invalid);
    assert!(result.is_err());
}

#[test]
fn test_include_patterns_deserialize_error() {
    let invalid = r#"["[invalid"]"#;
    let result: Result<IncludePatterns, _> = serde_json::from_str(invalid);
    assert!(result.is_err());
}

#[test]
fn test_patterns_with_special_chars() {
    let patterns = ExcludePatterns::new(vec![r"\$\d+".to_string()]).unwrap();
    assert!(patterns.matches("$100"));
    assert!(!patterns.matches("100"));

    let patterns = IncludePatterns::new(vec![r"[!@#$%^&*()]+".to_string()]).unwrap();
    assert!(patterns.matches("test@example.com"));
    assert!(!patterns.matches("testexample"));
}
