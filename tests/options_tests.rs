use word_tally::{Case, Format, Options};

#[test]
fn test_options_format_default() {
    let options = Options::default();
    assert_eq!(options.serialization().format(), Format::Text);
}

#[test]
fn test_options_with_format() {
    let options = Options::default().with_format(Format::Json);
    assert_eq!(options.serialization().format(), Format::Json);

    let options = Options::default()
        .with_format(Format::Csv)
        .with_case(Case::Upper);

    assert_eq!(options.serialization().format(), Format::Csv);
    assert_eq!(options.case(), Case::Upper);
}

#[test]
fn test_options_display_includes_format() {
    let options = Options::default().with_format(Format::Json);
    let display_string = options.to_string();

    assert!(display_string.contains("serialization:"));
    assert!(display_string.contains("format: json"));
}

#[test]
fn test_format_field_in_struct() {
    let options = Options::default();
    assert_eq!(options.serialization().format(), Format::Text);

    let options2 = Options::default().with_format(Format::Json);
    assert_eq!(options2.serialization().format(), Format::Json);
}

#[test]
fn test_format_serialization() {
    let options = Options::default().with_format(Format::Json);
    let serialized = serde_json::to_string(&options).unwrap();

    assert!(serialized.contains("\"format\":\"Json\""));

    let deserialized: Options = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.serialization().format(), Format::Json);
}
