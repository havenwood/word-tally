use std::collections::HashSet;

use word_tally::{ExcludeSet, IncludeSet};

#[test]
fn test_exclude_patterns_new() {
    let patterns = ExcludeSet::new(vec!["test".to_string()]).expect("process test");
    assert_eq!(patterns.len(), 1);
    assert!(!patterns.is_empty());
}

#[test]
fn test_exclude_patterns_empty() {
    let patterns = ExcludeSet::default();
    assert_eq!(patterns.len(), 0);
    assert!(patterns.is_empty());
}

#[test]
fn test_exclude_patterns_matches() {
    let patterns =
        ExcludeSet::new(vec!["^test".to_string(), "ing$".to_string()]).expect("process test");

    assert!(patterns.matches("testing"));
    assert!(patterns.matches("test"));
    assert!(patterns.matches("running"));
    assert!(!patterns.matches("run"));
}

#[test]
fn test_exclude_patterns_as_patterns() {
    let input = vec!["^pre".to_string(), "^un".to_string()];
    let patterns = ExcludeSet::new(input.clone()).expect("process test");
    assert_eq!(patterns.as_patterns(), input.as_slice());
}

#[test]
fn test_exclude_patterns_try_from() {
    let input = vec!["pattern1".to_string(), "pattern2".to_string()];
    let patterns = ExcludeSet::try_from(input.as_slice()).expect("process test");
    assert_eq!(patterns.len(), 2);
}

#[test]
fn test_exclude_patterns_invalid_regex() {
    let result = ExcludeSet::new(vec!["[invalid".to_string()]);
    assert!(result.is_err());
}

#[test]
fn test_include_patterns_new() {
    let patterns = IncludeSet::new(vec!["test".to_string()]).expect("process test");
    assert_eq!(patterns.len(), 1);
    assert!(!patterns.is_empty());
}

#[test]
fn test_include_patterns_empty() {
    let patterns = IncludeSet::default();
    assert_eq!(patterns.len(), 0);
    assert!(patterns.is_empty());
}

#[test]
fn test_include_patterns_matches() {
    let patterns = IncludeSet::new(vec!["[aeiou]".to_string()]).expect("process test");

    assert!(patterns.matches("test"));
    assert!(patterns.matches("hello"));
    assert!(!patterns.matches("rhythm"));
    assert!(!patterns.matches("xyz"));
}

#[test]
fn test_include_patterns_multiple() {
    let patterns =
        IncludeSet::new(vec!["^pre".to_string(), "^un".to_string()]).expect("process test");

    assert!(patterns.matches("prevent"));
    assert!(patterns.matches("unlike"));
    assert!(!patterns.matches("likely"));
    assert!(!patterns.matches("post"));
}

#[test]
fn test_include_patterns_as_patterns() {
    let input = vec!["test1".to_string(), "test2".to_string()];
    let patterns = IncludeSet::new(input.clone()).expect("process test");
    assert_eq!(patterns.as_patterns(), input.as_slice());
}

#[test]
fn test_include_patterns_try_from() {
    let input = vec!["^start".to_string(), "end$".to_string()];
    let patterns = IncludeSet::try_from(input.as_slice()).expect("process test");
    assert_eq!(patterns.len(), 2);
}

#[test]
fn test_include_patterns_invalid_regex() {
    let result = IncludeSet::new(vec!["[invalid".to_string()]);
    assert!(result.is_err());
}

#[test]
fn test_complex_regex_patterns() {
    let patterns = ExcludeSet::new(vec![r"\btest\b".to_string()]).expect("process test");
    assert!(patterns.matches("test"));
    assert!(!patterns.matches("testing"));

    let patterns = IncludeSet::new(vec![r"[0-9]+".to_string()]).expect("process test");
    assert!(patterns.matches("abc123"));
    assert!(!patterns.matches("abc"));

    let patterns = ExcludeSet::new(vec![r"^the$".to_string()]).expect("process test");
    assert!(patterns.matches("the"));
    assert!(!patterns.matches("them"));
    assert!(!patterns.matches("soothe"));
}

#[test]
fn test_exclude_patterns_as_ref() {
    let input = vec!["pattern".to_string()];
    let patterns = ExcludeSet::new(input.clone()).expect("process test");
    let as_ref: &[String] = patterns.as_ref();
    assert_eq!(as_ref, input.as_slice());
}

#[test]
fn test_include_patterns_as_ref() {
    let input = vec!["pattern".to_string()];
    let patterns = IncludeSet::new(input.clone()).expect("process test");
    let as_ref: &[String] = patterns.as_ref();
    assert_eq!(as_ref, input.as_slice());
}

#[test]
fn test_exclude_patterns_eq() {
    let patterns1 = ExcludeSet::new(vec!["test".to_string()]).expect("process test");
    let patterns2 = ExcludeSet::new(vec!["test".to_string()]).expect("process test");
    let patterns3 = ExcludeSet::new(vec!["other".to_string()]).expect("process test");

    assert_eq!(patterns1, patterns2);
    assert_ne!(patterns1, patterns3);
}

#[test]
fn test_include_patterns_eq() {
    let patterns1 = IncludeSet::new(vec!["test".to_string()]).expect("process test");
    let patterns2 = IncludeSet::new(vec!["test".to_string()]).expect("process test");
    let patterns3 = IncludeSet::new(vec!["other".to_string()]).expect("process test");

    assert_eq!(patterns1, patterns2);
    assert_ne!(patterns1, patterns3);
}

#[test]
fn test_exclude_patterns_ord() {
    let patterns1 = ExcludeSet::new(vec!["aaa".to_string()]).expect("process test");
    let patterns2 = ExcludeSet::new(vec!["bbb".to_string()]).expect("process test");

    assert!(patterns1 < patterns2);
    assert!(patterns2 > patterns1);
}

#[test]
fn test_include_patterns_ord() {
    let patterns1 = IncludeSet::new(vec!["aaa".to_string()]).expect("process test");
    let patterns2 = IncludeSet::new(vec!["bbb".to_string()]).expect("process test");

    assert!(patterns1 < patterns2);
    assert!(patterns2 > patterns1);
}

#[test]
// Testing hash implementation with interior mutability is safe here
#[allow(clippy::mutable_key_type)]
fn test_exclude_patterns_hash() {
    let patterns1 = ExcludeSet::new(vec!["test".to_string()]).expect("process test");
    let patterns2 = ExcludeSet::new(vec!["test".to_string()]).expect("process test");

    let mut set = HashSet::new();
    set.insert(patterns1);
    assert!(set.contains(&patterns2));
}

#[test]
// Testing hash implementation with interior mutability is safe here
#[allow(clippy::mutable_key_type)]
fn test_include_patterns_hash() {
    let patterns1 = IncludeSet::new(vec!["test".to_string()]).expect("process test");
    let patterns2 = IncludeSet::new(vec!["test".to_string()]).expect("process test");

    let mut set = HashSet::new();
    set.insert(patterns1);
    assert!(set.contains(&patterns2));
}

#[test]
fn test_exclude_patterns_display() {
    let patterns = ExcludeSet::new(vec!["pattern1".to_string(), "pattern2".to_string()])
        .expect("process test");
    assert_eq!(patterns.to_string(), "pattern1,pattern2");
}

#[test]
fn test_include_patterns_display() {
    let patterns = IncludeSet::new(vec!["pattern1".to_string(), "pattern2".to_string()])
        .expect("process test");
    assert_eq!(patterns.to_string(), "pattern1,pattern2");
}

#[test]
fn test_exclude_patterns_serde() {
    let patterns =
        ExcludeSet::new(vec!["test1".to_string(), "test2".to_string()]).expect("process test");

    let serialized = serde_json::to_string(&patterns).expect("serialize JSON");
    assert_eq!(serialized, r#"["test1","test2"]"#);

    let deserialized: ExcludeSet = serde_json::from_str(&serialized).expect("deserialize JSON");
    assert_eq!(patterns, deserialized);
}

#[test]
fn test_include_patterns_serde() {
    let patterns =
        IncludeSet::new(vec!["test1".to_string(), "test2".to_string()]).expect("process test");

    let serialized = serde_json::to_string(&patterns).expect("serialize JSON");
    assert_eq!(serialized, r#"["test1","test2"]"#);

    let deserialized: IncludeSet = serde_json::from_str(&serialized).expect("deserialize JSON");
    assert_eq!(patterns, deserialized);
}

#[test]
fn test_exclude_patterns_deserialize_error() {
    let invalid = r#"["[invalid"]"#;
    let result: Result<ExcludeSet, _> = serde_json::from_str(invalid);
    assert!(result.is_err());
}

#[test]
fn test_include_patterns_deserialize_error() {
    let invalid = r#"["[invalid"]"#;
    let result: Result<IncludeSet, _> = serde_json::from_str(invalid);
    assert!(result.is_err());
}

#[test]
fn test_patterns_with_special_chars() {
    let patterns = ExcludeSet::new(vec![r"\$\d+".to_string()]).expect("process test");
    assert!(patterns.matches("$100"));
    assert!(!patterns.matches("100"));

    let patterns = IncludeSet::new(vec![r"[!@#$%^&*()]+".to_string()]).expect("process test");
    assert!(patterns.matches("test@example.com"));
    assert!(!patterns.matches("testexample"));
}
