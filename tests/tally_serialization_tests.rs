use std::sync::Arc;
use word_tally::{Case, Input, Options, Serialization, Sort, WordTally};

fn make_shared<T>(value: T) -> Arc<T> {
    Arc::new(value)
}

#[test]
fn test_to_json() {
    let input_text = b"wombat wombat bat";
    let mut temp_file = tempfile::NamedTempFile::new().expect("create temp file");
    std::io::Write::write_all(&mut temp_file, input_text).expect("write test data");

    let options = Options::default();
    let shared_options = make_shared(options);

    let input = Input::new(
        temp_file.path().to_str().expect("temp file path"),
        shared_options.io(),
    )
    .expect("create input");

    let expected = WordTally::new(&input, &shared_options).expect("create word tally");
    let serialized = serde_json::to_string(&expected).expect("serialize JSON");

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
    let mut temp_file = tempfile::NamedTempFile::new().expect("create temp file");
    std::io::Write::write_all(&mut temp_file, input_text).expect("write test data");

    let options = Options::default();
    let input = Input::new(
        temp_file.path().to_str().expect("temp file path"),
        options.io(),
    )
    .expect("create input");

    let original = WordTally::new(&input, &options).expect("create word tally");
    let json = serde_json::to_string(&original).expect("serialize JSON");
    let deserialized: WordTally<'_> = serde_json::from_str(&json).expect("deserialize JSON");

    assert_eq!(deserialized.count(), original.count());
    assert_eq!(deserialized.uniq_count(), original.uniq_count());
    assert_eq!(deserialized.tally(), original.tally());
    assert_eq!(deserialized.options().case(), original.options().case());
    assert_eq!(deserialized.options().sort(), original.options().sort());
}

#[test]
fn test_json_field_renamed() {
    let input_text = b"test json field renaming";
    let mut temp_file = tempfile::NamedTempFile::new().expect("create temp file");
    std::io::Write::write_all(&mut temp_file, input_text).expect("write test data");

    let options = Options::default();
    let input = Input::new(
        temp_file.path().to_str().expect("temp file path"),
        options.io(),
    )
    .expect("create input");

    let original = WordTally::new(&input, &options).expect("create word tally");
    let json = serde_json::to_string(&original).expect("serialize JSON");

    assert!(json.contains("uniqueCount"));
    assert!(!json.contains("uniq_count"));
}

#[test]
fn test_deserialization_with_serde() {
    let input_text = b"wombat wombat bat";
    let mut temp_file = tempfile::NamedTempFile::new().expect("create temp file");
    std::io::Write::write_all(&mut temp_file, input_text).expect("write test data");

    let options = Options::default();
    let input = Input::new(
        temp_file.path().to_str().expect("temp file path"),
        options.io(),
    )
    .expect("create input");

    let original = WordTally::new(&input, &options).expect("create word tally");
    let json = serde_json::to_string(&original).expect("serialize JSON");
    let deserialized: WordTally<'_> = serde_json::from_str(&json).expect("deserialize JSON");

    assert_eq!(deserialized.count(), original.count());
    assert_eq!(deserialized.uniq_count(), original.uniq_count());
    assert_eq!(deserialized.tally(), original.tally());

    assert_eq!(deserialized.options().case(), original.options().case());
    assert_eq!(deserialized.options().sort(), original.options().sort());
}

#[test]
fn test_serialization_with_format() {
    let format_only = Serialization::Json;
    assert_eq!(format_only, Serialization::Json);
}

#[test]
fn test_serialization_with_field_delimiter() {
    let delim = Serialization::text().with_field_delimiter("::");
    assert_eq!(delim.field_delimiter(), Some("::"));
}

#[test]
fn test_deserialization_errors() {
    let invalid_json = "this is not json";
    let result: Result<WordTally<'_>, _> = serde_json::from_str(invalid_json);
    assert!(result.is_err());

    let missing_fields_json = r#"{
        "tally": [["test", 1]]
    }
    "#;

    let result: Result<WordTally<'_>, _> = serde_json::from_str(missing_fields_json);
    assert!(result.is_err());
}

#[test]
fn test_comprehensive_wordtally_serialization() {
    let content = b"apple banana apple cherry banana apple";
    let mut temp_file = tempfile::NamedTempFile::new().expect("create temp file");
    std::io::Write::write_all(&mut temp_file, content).expect("write test data");

    let options = Options::default()
        .with_case(Case::Lower)
        .with_sort(Sort::Desc);

    let input = Input::new(temp_file.path(), options.io()).expect("process test");
    let tally = WordTally::new(&input, &options).expect("create word tally");

    let json = serde_json::to_string(&tally).expect("serialize JSON");

    let deserialized: WordTally<'_> = serde_json::from_str(&json).expect("deserialize JSON");

    assert_eq!(tally.count(), deserialized.count());
    assert_eq!(tally.uniq_count(), deserialized.uniq_count());
    assert_eq!(tally.tally(), deserialized.tally());

    assert_eq!(tally.options().case(), deserialized.options().case());
    assert_eq!(tally.options().sort(), deserialized.options().sort());
}

#[test]
fn test_json_field_names() {
    let content = b"test words for json";
    let mut temp_file = tempfile::NamedTempFile::new().expect("create temp file");
    std::io::Write::write_all(&mut temp_file, content).expect("write test data");

    let options = Options::default();
    let input = Input::new(temp_file.path(), options.io()).expect("process test");
    let tally = WordTally::new(&input, &options).expect("create word tally");

    let json = serde_json::to_string(&tally).expect("serialize JSON");

    assert!(json.contains("\"uniqueCount\""));
    assert!(!json.contains("\"uniq_count\""));

    assert!(json.contains("\"options\""));
    assert!(json.contains("\"tally\""));
}

#[test]
fn test_round_trip_serialization() {
    let content = b"one two three one two one";
    let mut temp_file = tempfile::NamedTempFile::new().expect("create temp file");
    std::io::Write::write_all(&mut temp_file, content).expect("write test data");

    let options = Options::default()
        .with_case(Case::Upper)
        .with_sort(Sort::Asc)
        .with_serialization(Serialization::Json)
        .with_filters(word_tally::Filters::default().with_min_chars(2));

    let input = Input::new(temp_file.path(), options.io()).expect("process test");
    let original = WordTally::new(&input, &options).expect("create word tally");

    let json = serde_json::to_string(&original).expect("serialize JSON");

    let deserialized: WordTally<'_> = serde_json::from_str(&json).expect("deserialize JSON");

    assert_eq!(original.count(), deserialized.count());
    assert_eq!(original.uniq_count(), deserialized.uniq_count());
    assert_eq!(original.tally(), deserialized.tally());

    assert_eq!(original.options().case(), deserialized.options().case());
    assert_eq!(original.options().sort(), deserialized.options().sort());
    assert_eq!(
        original.options().serialization(),
        deserialized.options().serialization()
    );
    assert_eq!(
        original.options().filters().min_chars(),
        deserialized.options().filters().min_chars()
    );
}
