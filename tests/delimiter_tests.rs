//! Tests for the Delimiter type.

use word_tally::options::delimiter::Delimiter;

#[test]
fn test_delimiter_from_literal() {
    let delim = Delimiter::from_literal(" ");
    assert_eq!(delim.as_str(), " ");

    let delim = Delimiter::from_literal("\t");
    assert_eq!(delim.as_str(), "\t");
}

#[test]
fn test_delimiter_from_escaped() {
    // Basic escape sequences
    assert_eq!(Delimiter::from_escaped("\\0").as_str(), "\0");
    assert_eq!(Delimiter::from_escaped("\\t").as_str(), "\t");
    assert_eq!(Delimiter::from_escaped("\\n").as_str(), "\n");
    assert_eq!(Delimiter::from_escaped("\\r").as_str(), "\r");
    assert_eq!(Delimiter::from_escaped("\\\\").as_str(), "\\");
    assert_eq!(Delimiter::from_escaped("\\\"").as_str(), "\"");
    assert_eq!(Delimiter::from_escaped("\\'").as_str(), "'");

    // No escapes
    assert_eq!(Delimiter::from_escaped(" ").as_str(), " ");
    assert_eq!(Delimiter::from_escaped(",").as_str(), ",");
    assert_eq!(Delimiter::from_escaped("::").as_str(), "::");

    // Unknown escape sequences (just use the character)
    assert_eq!(Delimiter::from_escaped("\\x").as_str(), "x");
    assert_eq!(Delimiter::from_escaped("\\a").as_str(), "a");
}

#[test]
fn test_delimiter_from_escaped_error() {
    // Trailing backslash is kept as literal backslash
    assert_eq!(Delimiter::from_escaped("\\").as_str(), "\\");
    assert_eq!(Delimiter::from_escaped("test\\").as_str(), "test\\");
}

#[test]
fn test_delimiter_display() {
    // Display should show quoted form
    assert_eq!(format!("{}", Delimiter::from_literal("\t")), "\"\\t\"");
    assert_eq!(format!("{}", Delimiter::from_literal("\n")), "\"\\n\"");
    assert_eq!(format!("{}", Delimiter::from_literal(" ")), "\" \"");
}

#[test]
fn test_delimiter_serialization() {
    let delim = Delimiter::from_literal("\t");
    let json = serde_json::to_string(&delim).expect("serialize JSON");
    assert_eq!(json, r#""\t""#); // Should serialize as actual tab, not escaped

    let deserialized: Delimiter = serde_json::from_str(&json).expect("deserialize JSON");
    assert_eq!(deserialized.as_str(), "\t");
}

#[test]
fn test_delimiter_equality() {
    let d1 = Delimiter::from_literal(" ");
    let d2 = Delimiter::from_literal(" ");
    let d3 = Delimiter::from_literal("\t");

    assert_eq!(d1, d2);
    assert_ne!(d1, d3);
}

#[test]
#[allow(clippy::redundant_clone)]
// We're specifically testing clone functionality
fn test_delimiter_clone() {
    let d1 = Delimiter::from_literal("||");
    let d2 = d1.clone();
    assert_eq!(d1, d2);
    assert_eq!(d1.as_str(), d2.as_str());
}

#[test]
fn test_delimiter_empty() {
    let delim = Delimiter::from_literal("");
    assert_eq!(delim.as_str(), "");
    assert_eq!(delim.display_quoted(), "\"\"");
}

#[test]
fn test_delimiter_unicode() {
    let delim = Delimiter::from_literal("→");
    assert_eq!(delim.as_str(), "→");
    assert_eq!(delim.display_quoted(), "\"→\"");

    let delim = Delimiter::from_escaped("→");
    assert_eq!(delim.as_str(), "→");
}

#[test]
fn test_delimiter_default() {
    let delim = Delimiter::default();
    assert_eq!(delim.as_str(), Delimiter::DEFAULT_FIELD);
}

#[test]
fn test_delimiter_from_string() {
    let delim = Delimiter::from("test".to_string());
    assert_eq!(delim.as_str(), "test");

    let delim = Delimiter::from("another");
    assert_eq!(delim.as_str(), "another");
}

#[test]
fn test_delimiter_as_ref() {
    let delim = Delimiter::from_literal("test");
    let s: &str = delim.as_ref();
    assert_eq!(s, "test");
}
