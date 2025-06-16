//! Tests for the `Delimiters` type.

use word_tally::options::delimiters::Delimiters;

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
