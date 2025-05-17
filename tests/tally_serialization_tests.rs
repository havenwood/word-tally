use std::sync::Arc;
use word_tally::{Format, Input, Options, Serialization, WordTally};

fn make_shared<T>(value: T) -> Arc<T> {
    Arc::new(value)
}

#[test]
fn test_to_json() {
    let input_text = b"wombat wombat bat";
    let mut temp_file = tempfile::NamedTempFile::new().unwrap();
    std::io::Write::write_all(&mut temp_file, input_text).unwrap();

    let options = Options::default();
    let shared_options = make_shared(options);

    let input = Input::new(temp_file.path().to_str().unwrap(), shared_options.io())
        .expect("Failed to create Input");

    let expected = WordTally::new(&input, &shared_options).expect("Failed to create WordTally");
    let serialized = serde_json::to_string(&expected).unwrap();

    assert!(serialized.contains("\"tally\":[[\"wombat\",2],[\"bat\",1]]"));
    assert!(serialized.contains("\"count\":3"));
    assert!(serialized.contains("\"uniqueCount\":2"));
    assert!(!serialized.contains("\"uniq_count\":"));
    assert!(serialized.contains("\"options\":"));
    assert!(serialized.contains("\"filters\":"));
}

#[test]
fn test_from_json() {
    let input_text = b"wombat wombat bat";
    let mut temp_file = tempfile::NamedTempFile::new().unwrap();
    std::io::Write::write_all(&mut temp_file, input_text).unwrap();

    let options = Options::default();
    let input = Input::new(temp_file.path().to_str().unwrap(), options.io())
        .expect("Failed to create Input");

    let original = WordTally::new(&input, &options).expect("Failed to create WordTally");
    let json = serde_json::to_string(&original).unwrap();
    let deserialized: WordTally<'_> = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.count(), original.count());
    assert_eq!(deserialized.uniq_count(), original.uniq_count());
    assert_eq!(deserialized.tally(), original.tally());
    assert_eq!(deserialized.options().case(), original.options().case());
    assert_eq!(deserialized.options().sort(), original.options().sort());
}

#[test]
fn test_json_field_renamed() {
    let input_text = b"test json field renaming";
    let mut temp_file = tempfile::NamedTempFile::new().unwrap();
    std::io::Write::write_all(&mut temp_file, input_text).unwrap();

    let options = Options::default();
    let input = Input::new(temp_file.path().to_str().unwrap(), options.io())
        .expect("Failed to create Input");

    let original = WordTally::new(&input, &options).expect("Failed to create WordTally");
    let json = serde_json::to_string(&original).unwrap();

    assert!(json.contains("uniqueCount"));
    assert!(!json.contains("uniq_count"));
}

#[test]
fn test_deserialization_with_serde() {
    let input_text = b"wombat wombat bat";
    let mut temp_file = tempfile::NamedTempFile::new().unwrap();
    std::io::Write::write_all(&mut temp_file, input_text).unwrap();

    let options = Options::default();
    let input = Input::new(temp_file.path().to_str().unwrap(), options.io())
        .expect("Failed to create Input");

    let original = WordTally::new(&input, &options).expect("Failed to create WordTally");
    let json = serde_json::to_string(&original).unwrap();
    let deserialized: WordTally<'_> = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.count(), original.count());
    assert_eq!(deserialized.uniq_count(), original.uniq_count());
    assert_eq!(deserialized.tally(), original.tally());

    // Options should be functionally equivalent but not the same instance
    assert_eq!(deserialized.options().case(), original.options().case());
    assert_eq!(deserialized.options().sort(), original.options().sort());
    assert!(!std::ptr::eq(deserialized.options(), original.options()));
}

#[test]
fn test_serialization_with_format() {
    let format_only = Serialization::with_format(Format::Json);
    assert_eq!(format_only.format(), Format::Json);
}

#[test]
fn test_serialization_with_delimiter() {
    let delim = Serialization::with_delimiter("::").unwrap();
    assert_eq!(delim.delimiter(), "::");
}

#[test]
fn test_error_handling_invalid_json() {
    // Invalid JSON syntax
    let invalid_json = r#"
    {
        "tally": [["test", 1]],
        "count": 1,
        "uniqueCount": 1,
        this is invalid
    }
    "#;

    let result: Result<WordTally<'_>, _> = serde_json::from_str(invalid_json);
    assert!(result.is_err());
}

#[test]
fn test_error_handling_missing_fields() {
    // Missing required fields
    let missing_fields_json = r#"
    {
        "tally": [["test", 1]]
    }
    "#;

    let result: Result<WordTally<'_>, _> = serde_json::from_str(missing_fields_json);
    assert!(result.is_err());
}
