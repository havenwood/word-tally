//! Tests for serialization functionality.

use word_tally::{Delimiters, Serialization};

#[test]
fn test_serialization_text() {
    let serialization = Serialization::Text(Delimiters::default());
    assert!(matches!(serialization, Serialization::Text(_)));
    assert_eq!(serialization.field_delimiter_display(), "\" \"");
}

#[test]
fn test_serialization_json() {
    let serialization = Serialization::Json;
    assert!(matches!(serialization, Serialization::Json));
    assert_eq!(serialization.field_delimiter_display(), "n/a");
}

#[test]
fn test_serialization_csv() {
    let serialization = Serialization::Csv;
    assert!(matches!(serialization, Serialization::Csv));
    assert_eq!(serialization.field_delimiter_display(), "n/a");
}

#[test]
fn test_serialization_text_with_tab() {
    let delimiters = Delimiters::default().with_field_delimiter("\t");
    let serialization = Serialization::Text(delimiters);
    assert_eq!(serialization.field_delimiter_display(), "\"\\t\"");
}

#[test]
fn test_serialization_default() {
    let serialization = Serialization::default();
    assert!(matches!(serialization, Serialization::Text(_)));
    assert_eq!(serialization.field_delimiter_display(), "\" \"");
}

#[test]
fn test_serialization_text_with_entry_delimiter() {
    let delimiters = Delimiters::default()
        .with_field_delimiter("\t")
        .with_entry_delimiter(";");
    let serialization = Serialization::Text(delimiters);
    assert_eq!(serialization.field_delimiter_display(), "\"\\t\"");
    assert_eq!(serialization.entry_delimiter_display(), "\";\"");
}

#[test]
fn test_serialization_entry_delimiter_json_csv() {
    assert_eq!(Serialization::Json.entry_delimiter_display(), "n/a");
    assert_eq!(Serialization::Csv.entry_delimiter_display(), "n/a");
}

#[test]
fn test_with_field_delimiter_preserves_entry_delimiter() {
    let delimiters = Delimiters::default()
        .with_entry_delimiter(";;")
        .with_field_delimiter("::");
    let serialization = Serialization::Text(delimiters);
    assert_eq!(serialization.field_delimiter_display(), "\"::\"");
    assert_eq!(serialization.entry_delimiter_display(), "\";;\"");
}

#[test]
fn test_with_entry_delimiter_preserves_delimiter() {
    let delimiters = Delimiters::default()
        .with_field_delimiter("::")
        .with_entry_delimiter(";;");
    let serialization = Serialization::Text(delimiters);

    assert_eq!(serialization.field_delimiter_display(), "\"::\"");
    assert_eq!(serialization.entry_delimiter_display(), "\";;\"");
}

#[test]
fn test_chaining_multiple_delimiter_changes() {
    let delimiters = Delimiters::default()
        .with_field_delimiter("first")
        .with_entry_delimiter("entry1")
        .with_field_delimiter("second")
        .with_entry_delimiter("entry2");
    let serialization = Serialization::Text(delimiters);

    assert_eq!(serialization.field_delimiter_display(), "\"second\"");
    assert_eq!(serialization.entry_delimiter_display(), "\"entry2\"");
}

#[test]
fn test_non_text_variants_have_no_delimiters() {
    // JSON and CSV variants have no delimiter configuration
    assert!(matches!(Serialization::Json, Serialization::Json));
    assert!(matches!(Serialization::Csv, Serialization::Csv));

    // Display methods return "n/a" for non-text formats
    assert_eq!(Serialization::Json.field_delimiter_display(), "n/a");
    assert_eq!(Serialization::Json.entry_delimiter_display(), "n/a");
    assert_eq!(Serialization::Csv.field_delimiter_display(), "n/a");
    assert_eq!(Serialization::Csv.entry_delimiter_display(), "n/a");
}
