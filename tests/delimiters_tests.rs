//! Tests for the `Delimiters` and `Delimiter` types.

use word_tally::options::delimiters::{Delimiter, Delimiters};

#[test]
fn test_delimiters_new() {
    let delimiters = Delimiters::new(" ", "\n");
    assert_eq!(delimiters.field(), " ");
    assert_eq!(delimiters.entry(), "\n");

    // Test with literal tab and newline
    let delimiters = Delimiters::new("\t", "\r\n");
    assert_eq!(delimiters.field(), "\t");
    assert_eq!(delimiters.entry(), "\r\n");

    // Test that strings are NOT unescaped
    let delimiters = Delimiters::new("\\t", "\\n");
    assert_eq!(delimiters.field(), "\\t");
    assert_eq!(delimiters.entry(), "\\n");
}

#[test]
fn test_delimiters_default() {
    let delimiters = Delimiters::default();
    assert_eq!(delimiters.field(), Delimiters::DEFAULT_FIELD);
    assert_eq!(delimiters.entry(), Delimiters::DEFAULT_ENTRY);
}

#[test]
fn test_delimiters_constants() {
    assert_eq!(Delimiters::DEFAULT_FIELD, " ");
    assert_eq!(Delimiters::DEFAULT_ENTRY, "\n");
}

#[test]
fn test_delimiters_with_field_delimiter() {
    let delimiters = Delimiters::default().with_field_delimiter("::");
    assert_eq!(delimiters.field(), "::");
    assert_eq!(delimiters.entry(), "\n");
}

#[test]
fn test_delimiters_with_entry_delimiter() {
    let delimiters = Delimiters::default().with_entry_delimiter(";;");
    assert_eq!(delimiters.field(), " ");
    assert_eq!(delimiters.entry(), ";;");
}

#[test]
fn test_delimiters_with_literal_delimiters() {
    let delimiters = Delimiters::default()
        .with_field_delimiter("\t")
        .with_entry_delimiter("\r\n");
    assert_eq!(delimiters.field(), "\t");
    assert_eq!(delimiters.entry(), "\r\n");
}

#[test]
fn test_delimiters_chaining() {
    let delimiters = Delimiters::default()
        .with_field_delimiter("first")
        .with_entry_delimiter("second")
        .with_field_delimiter("third");
    assert_eq!(delimiters.field(), "third");
    assert_eq!(delimiters.entry(), "second");
}

#[test]
fn test_delimiters_field_display() {
    let delimiters = Delimiters::default().with_field_delimiter("\t");
    assert_eq!(delimiters.field_display(), "\"\\t\"");

    let delimiters = Delimiters::default().with_field_delimiter(" ");
    assert_eq!(delimiters.field_display(), "\" \"");
}

#[test]
fn test_delimiters_entry_display() {
    let delimiters = Delimiters::default().with_entry_delimiter("\n");
    assert_eq!(delimiters.entry_display(), "\"\\n\"");

    let delimiters = Delimiters::default().with_entry_delimiter("||");
    assert_eq!(delimiters.entry_display(), "\"||\"");
}

#[test]
fn test_delimiters_display() {
    let delimiters = Delimiters::default();
    assert_eq!(format!("{delimiters}"), "field=\" \", entry=\"\\n\"");

    let delimiters = Delimiters::default()
        .with_field_delimiter("\t")
        .with_entry_delimiter(";;");
    assert_eq!(format!("{delimiters}"), "field=\"\\t\", entry=\";;\"");
}

#[test]
#[allow(clippy::redundant_clone)]
// We're specifically testing clone functionality
fn test_delimiters_clone() {
    let delimiters1 = Delimiters::default()
        .with_field_delimiter("::")
        .with_entry_delimiter("||");
    let delimiters2 = delimiters1.clone();
    assert_eq!(delimiters1.field(), delimiters2.field());
    assert_eq!(delimiters1.entry(), delimiters2.entry());
}

#[test]
fn test_delimiters_equality() {
    let delimiters1 = Delimiters::default().with_field_delimiter("::");
    let delimiters2 = Delimiters::default().with_field_delimiter("::");
    let delimiters3 = Delimiters::default().with_field_delimiter("||");

    assert_eq!(delimiters1, delimiters2);
    assert_ne!(delimiters1, delimiters3);
}

#[test]
fn test_delimiters_serialization() {
    let delimiters = Delimiters::default()
        .with_field_delimiter("\t")
        .with_entry_delimiter("\r\n");

    let json = serde_json::to_string(&delimiters).expect("serialize JSON");
    let expected = r#"{"field":"\t","entry":"\r\n"}"#;
    assert_eq!(json, expected);

    let deserialized: Delimiters = serde_json::from_str(&json).expect("deserialize JSON");
    assert_eq!(deserialized.field(), "\t");
    assert_eq!(deserialized.entry(), "\r\n");
}

#[test]
fn test_delimiters_hash() {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let delimiters1 = Delimiters::default().with_field_delimiter("::");
    let delimiters2 = Delimiters::default().with_field_delimiter("::");

    let mut hasher1 = DefaultHasher::new();
    delimiters1.hash(&mut hasher1);
    let hash1 = hasher1.finish();

    let mut hasher2 = DefaultHasher::new();
    delimiters2.hash(&mut hasher2);
    let hash2 = hasher2.finish();

    assert_eq!(hash1, hash2);
}

#[test]
fn test_delimiters_ord() {
    let delimiters1 = Delimiters::default().with_field_delimiter("a");
    let delimiters2 = Delimiters::default().with_field_delimiter("b");

    assert!(delimiters1 < delimiters2);
    assert!(delimiters2 > delimiters1);
}

#[test]
fn test_delimiters_empty_delimiters() {
    let delimiters = Delimiters::default()
        .with_field_delimiter("")
        .with_entry_delimiter("");
    assert_eq!(delimiters.field(), "");
    assert_eq!(delimiters.entry(), "");
    assert_eq!(delimiters.field_display(), "\"\"");
    assert_eq!(delimiters.entry_display(), "\"\"");
}

#[test]
fn test_delimiters_unicode() {
    let delimiters = Delimiters::default()
        .with_field_delimiter("→")
        .with_entry_delimiter("❯");
    assert_eq!(delimiters.field(), "→");
    assert_eq!(delimiters.entry(), "❯");
}

#[test]
fn test_delimiters_preserves_literal_backslash() {
    let delimiters = Delimiters::default()
        .with_field_delimiter("\\")
        .with_entry_delimiter("\\\\");
    assert_eq!(delimiters.field(), "\\");
    assert_eq!(delimiters.entry(), "\\\\");
}

#[test]
fn test_delimiters_debug() {
    let delimiters = Delimiters::default()
        .with_field_delimiter("::")
        .with_entry_delimiter("||");
    let debug_str = format!("{delimiters:?}");
    assert!(debug_str.contains("Delimiters"));
    assert!(debug_str.contains("field"));
    assert!(debug_str.contains("entry"));
}

#[test]
fn test_delimiters_serde_field_names() {
    let delimiters = Delimiters::default()
        .with_field_delimiter("::")
        .with_entry_delimiter("||");

    let json = serde_json::to_string(&delimiters).expect("serialize JSON");
    assert!(json.contains("\"field\":"));
    assert!(json.contains("\"entry\":"));
    assert!(!json.contains("\"field_delimiter\":"));
    assert!(!json.contains("\"entry_delimiter\":"));
}

// Tests for `Delimiter` type

#[test]
fn test_delimiter_from_str() {
    let delim = Delimiter::from(" ");
    assert_eq!(&*delim, " ");

    let delim = Delimiter::from("\t");
    assert_eq!(&*delim, "\t");
}

#[test]
fn test_delimiter_display() {
    let test_cases = [("\t", "\"\\t\""), ("\n", "\"\\n\""), (" ", "\" \"")];

    for (input, expected) in test_cases {
        assert_eq!(format!("{}", Delimiter::from(input)), expected);
    }
}

#[test]
fn test_delimiter_serialization() {
    let delim = Delimiter::from("\t");
    let json = serde_json::to_string(&delim).expect("serialize JSON");
    assert_eq!(json, r#""\t""#); // Should serialize as actual tab, not escaped

    let deserialized: Delimiter = serde_json::from_str(&json).expect("deserialize JSON");
    assert_eq!(&*deserialized, "\t");
}

#[test]
fn test_delimiter_equality() {
    let d1 = Delimiter::from(" ");
    let d2 = Delimiter::from(" ");
    let d3 = Delimiter::from("\t");

    assert_eq!(d1, d2);
    assert_ne!(d1, d3);
}

#[test]
#[allow(clippy::redundant_clone)]
// We're specifically testing clone functionality
fn test_delimiter_clone() {
    let d1 = Delimiter::from("||");
    let d2 = d1.clone();
    assert_eq!(d1, d2);
    assert_eq!(d1, d2);
}

#[test]
fn test_delimiter_empty() {
    let delim = Delimiter::from("");
    assert_eq!(&*delim, "");
    assert_eq!(delim.to_string(), "\"\"");
}

#[test]
fn test_delimiter_unicode() {
    let delim = Delimiter::from("→");
    assert_eq!(&*delim, "→");
    assert_eq!(delim.to_string(), "\"→\"");
}

#[test]
fn test_delimiter_from_string() {
    let delim = Delimiter::from("test".to_string());
    assert_eq!(&*delim, "test");

    let delim = Delimiter::from("another");
    assert_eq!(&*delim, "another");
}

#[test]
fn test_delimiter_deref() {
    let delim = Delimiter::from("test");
    assert_eq!(&*delim, "test");
    assert_eq!(delim.len(), 4);
    assert!(delim.starts_with("te"));
}

#[test]
fn test_delimiter_new() {
    assert_eq!(&*Delimiter::new("\t"), "\t");
    assert_eq!(&*Delimiter::new("\n"), "\n");
    assert_eq!(&*Delimiter::new("test"), "test");
}
