use word_tally::{ExcludeWords, Format, Input, Options, Output, Serialization, WordTally};

#[test]
fn test_input_default() {
    let input = Input::default();
    assert!(matches!(input, Input::Stdin));
}

#[test]
fn test_output_default() {
    let _output = Output::default();
    // Just verify it compiles
}

#[test]
fn test_words_exclude_from() {
    let words = vec!["beep".to_string(), "boop".to_string()];
    assert_eq!(ExcludeWords::from(words.clone()), ExcludeWords(words));
}

#[test]
fn test_with_defaults() {
    let (_temp_dir, file_path) = create_test_file();
    let options = Options::default();
    let input = Input::new(&file_path, options.io()).expect("Failed to create Input");
    let tally = WordTally::new(&input, &options).expect("Failed to create WordTally");
    assert_eq!(tally.count(), 3);
}

#[test]
fn test_with_parallel_processing() {
    let (_temp_dir, file_path) = create_test_file();
    let options = Options::default().with_processing(word_tally::Processing::Parallel);
    let input = Input::new(&file_path, options.io()).expect("Failed to create Input");
    let tally = WordTally::new(&input, &options).expect("Failed to create WordTally");
    assert_eq!(tally.count(), 3);
}

#[test]
fn test_with_custom_chunk_size() {
    let (_temp_dir, file_path) = create_test_file();
    let options = Options::default()
        .with_processing(word_tally::Processing::Parallel)
        .with_chunk_size(32_768);

    let input = Input::new(&file_path, options.io()).expect("Failed to create Input");
    let tally = WordTally::new(&input, &options).expect("Failed to create WordTally");
    assert_eq!(tally.count(), 3);
}

#[test]
fn test_serialization_with_format() {
    let format_only = Serialization::with_format(Format::Json);
    assert_eq!(format_only.format(), Format::Json);
}

#[test]
fn test_serialization_with_delimiter() {
    let delim = Serialization::with_delimiter("::").expect("create delimiter");
    assert_eq!(delim.delimiter(), "::");
}

const TEST_INPUT: &[u8] = b"test convenience constructors";

fn create_test_file() -> (tempfile::TempDir, String) {
    let temp_dir = tempfile::tempdir().expect("process test");
    let file_path = temp_dir.path().join("test_input.txt");
    std::fs::write(&file_path, TEST_INPUT).expect("process test");
    (
        temp_dir,
        file_path.to_str().expect("process test").to_string(),
    )
}
