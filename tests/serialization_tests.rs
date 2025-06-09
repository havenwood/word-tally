//! Tests for serialization functionality.

use word_tally::Serialization;

#[test]
fn test_serialization_text() {
    let serialization = Serialization::text();
    assert!(matches!(
        &serialization,
        Serialization::Text { field_delimiter, .. } if field_delimiter.as_str() == " "
    ));
    assert_eq!(serialization.field_delimiter(), Some(" "));
}

#[test]
fn test_serialization_json() {
    let serialization = Serialization::Json;
    assert!(matches!(serialization, Serialization::Json));
    assert_eq!(serialization.field_delimiter(), None);
}

#[test]
fn test_serialization_csv() {
    let serialization = Serialization::Csv;
    assert!(matches!(serialization, Serialization::Csv));
    assert_eq!(serialization.field_delimiter(), None);
}

#[test]
fn test_serialization_text_with_tab() {
    let serialization = Serialization::text().with_field_delimiter("\t");
    assert_eq!(serialization.field_delimiter(), Some("\t"));
}

#[test]
fn test_serialization_default() {
    let serialization = Serialization::text();
    assert!(matches!(
        &serialization,
        Serialization::Text { field_delimiter, .. } if field_delimiter.as_str() == " "
    ));
}

#[test]
fn test_serialization_text_with_entry_delimiter() {
    let serialization = Serialization::text()
        .with_field_delimiter("\t")
        .with_entry_delimiter(";");
    assert!(matches!(
        &serialization,
        Serialization::Text { field_delimiter, entry_delimiter }
        if field_delimiter.as_str() == "\t" && entry_delimiter.as_str() == ";"
    ));
    assert_eq!(serialization.field_delimiter(), Some("\t"));
    assert_eq!(serialization.entry_delimiter(), Some(";"));
}

#[test]
fn test_serialization_entry_delimiter_json_csv() {
    assert_eq!(Serialization::Json.entry_delimiter(), None);
    assert_eq!(Serialization::Csv.entry_delimiter(), None);
}

#[test]
fn test_with_field_delimiter_preserves_entry_delimiter() {
    let serialization = Serialization::text()
        .with_entry_delimiter(";;")
        .with_field_delimiter("::");

    assert!(matches!(
        &serialization,
        Serialization::Text { field_delimiter, entry_delimiter }
        if field_delimiter.as_str() == "::" && entry_delimiter.as_str() == ";;"
    ));
}

#[test]
fn test_with_entry_delimiter_preserves_delimiter() {
    let serialization = Serialization::text()
        .with_field_delimiter("::")
        .with_entry_delimiter(";;");

    assert!(matches!(
        &serialization,
        Serialization::Text { field_delimiter, entry_delimiter }
        if field_delimiter.as_str() == "::" && entry_delimiter.as_str() == ";;"
    ));
}

#[test]
fn test_chaining_multiple_delimiter_changes() {
    let serialization = Serialization::text()
        .with_field_delimiter("first")
        .with_entry_delimiter("entry1")
        .with_field_delimiter("second")
        .with_entry_delimiter("entry2");

    assert!(matches!(
        &serialization,
        Serialization::Text { field_delimiter, entry_delimiter }
        if field_delimiter.as_str() == "second" && entry_delimiter.as_str() == "entry2"
    ));
}

#[test]
fn test_with_field_delimiter_on_non_text_variants() {
    let json = Serialization::Json.with_field_delimiter("::");
    assert!(matches!(json, Serialization::Json));

    let csv = Serialization::Csv.with_entry_delimiter(";;");
    assert!(matches!(csv, Serialization::Csv));
}
