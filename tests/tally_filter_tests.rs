use std::{io::Write, sync::Arc};

use word_tally::{Case, Filters, Io, Options, Serialization, Sort, TallyMap, WordTally};

fn make_shared<T>(value: T) -> Arc<T> {
    Arc::new(value)
}

#[test]
fn test_excluding_words() {
    let input_text = "The tree that would grow to heaven must send its roots to hell.".as_bytes();
    let mut temp_file = tempfile::NamedTempFile::new().expect("create temp file");
    Write::write_all(&mut temp_file, input_text).expect("write test data");

    let words = vec!["heaven".to_string(), "hell".to_string()];
    let serializer = Serialization::default();
    let filters = Filters::default().with_exclude_words(words);
    let options = Options::new(
        Case::default(),
        Sort::Unsorted,
        serializer,
        filters,
        Io::ParallelStream,
        word_tally::Performance::default(),
    );
    let options_arc = make_shared(options);

    let tally_map = TallyMap::from_buffered_input(
        temp_file.path().to_str().expect("temp file path"),
        &options_arc,
    )
    .expect("create tally map");
    let tally = WordTally::from_tally_map(tally_map, &options_arc);
    let result = tally.tally();

    assert!(result.iter().any(|(word, _)| word.as_ref() == "tree"));
    assert!(!result.iter().any(|(word, _)| word.as_ref() == "heaven"));
    assert!(!result.iter().any(|(word, _)| word.as_ref() == "hell"));
}

#[test]
fn test_excluding_patterns() {
    let input_text = "The tree that would grow to heaven must send its roots to hell.".as_bytes();
    let mut temp_file = tempfile::NamedTempFile::new().expect("create temp file");
    Write::write_all(&mut temp_file, input_text).expect("write test data");

    let serializer = Serialization::default();

    // Create patterns to exclude words starting with 't'
    let patterns = vec!["^t.*".to_string()];
    let filters = Filters::default()
        .with_exclude_patterns(&patterns)
        .expect("set exclude patterns");

    let options = Options::new(
        Case::default(),
        Sort::Unsorted,
        serializer,
        filters,
        Io::ParallelStream,
        word_tally::Performance::default(),
    );
    let options_arc = make_shared(options);

    let tally_map = TallyMap::from_buffered_input(
        temp_file.path().to_str().expect("temp file path"),
        &options_arc,
    )
    .expect("create tally map");
    let tally = WordTally::from_tally_map(tally_map, &options_arc);
    let result = tally.tally();

    // These should be present
    assert!(result.iter().any(|(word, _)| word.as_ref() == "heaven"));
    assert!(result.iter().any(|(word, _)| word.as_ref() == "hell"));
    assert!(result.iter().any(|(word, _)| word.as_ref() == "would"));

    // These should be excluded by the pattern
    assert!(!result.iter().any(|(word, _)| word.as_ref() == "tree"));
    assert!(!result.iter().any(|(word, _)| word.as_ref() == "that"));
    assert!(!result.iter().any(|(word, _)| word.as_ref() == "to"));
}

#[test]
fn test_including_patterns() {
    let input_text = "The tree that would grow to heaven must send its roots to hell.".as_bytes();
    let mut temp_file = tempfile::NamedTempFile::new().expect("create temp file");
    Write::write_all(&mut temp_file, input_text).expect("write test data");
    let file_path = temp_file.path().to_str().expect("temp file path");

    let serializer = Serialization::default();

    // Create patterns to include only words starting with 'h'
    let patterns = vec!["^h.*".to_string()];
    let filters = Filters::default()
        .with_include_patterns(&patterns)
        .expect("set include patterns");

    let options = Options::new(
        Case::default(),
        Sort::Unsorted,
        serializer,
        filters,
        Io::ParallelStream,
        word_tally::Performance::default(),
    );
    let options_arc = make_shared(options);

    let tally_map =
        TallyMap::from_buffered_input(file_path, &options_arc).expect("create tally map");
    let tally = WordTally::from_tally_map(tally_map, &options_arc);
    let result = tally.tally();

    // These should be present (words starting with 'h')
    assert!(result.iter().any(|(word, _)| word.as_ref() == "heaven"));
    assert!(result.iter().any(|(word, _)| word.as_ref() == "hell"));

    // These should be excluded (words not starting with 'h')
    assert!(!result.iter().any(|(word, _)| word.as_ref() == "tree"));
    assert!(!result.iter().any(|(word, _)| word.as_ref() == "would"));
    assert!(!result.iter().any(|(word, _)| word.as_ref() == "to"));
    assert!(!result.iter().any(|(word, _)| word.as_ref() == "the"));

    // Make sure we only have words starting with 'h'
    assert!(result.iter().all(|(word, _)| word.starts_with('h')));
}

#[test]
fn test_combining_include_exclude_patterns() {
    let input_text = "The tree that would grow to heaven must send its roots to hell.".as_bytes();
    let mut temp_file = tempfile::NamedTempFile::new().expect("create temp file");
    Write::write_all(&mut temp_file, input_text).expect("write test data");
    let file_path = temp_file.path().to_str().expect("temp file path");

    let serializer = Serialization::default();

    // Include words starting with 'h' but exclude 'hell'
    let include_patterns = vec!["^h.*".to_string()];
    let exclude_patterns = vec!["^hell$".to_string()];

    let filters = Filters::default()
        .with_include_patterns(&include_patterns)
        .expect("execute operation")
        .with_exclude_patterns(&exclude_patterns)
        .expect("execute operation");

    let options = Options::new(
        Case::default(),
        Sort::Unsorted,
        serializer,
        filters,
        Io::ParallelStream,
        word_tally::Performance::default(),
    );
    let options_arc = make_shared(options);

    let tally_map =
        TallyMap::from_buffered_input(file_path, &options_arc).expect("create tally map");
    let tally = WordTally::from_tally_map(tally_map, &options_arc);
    let result = tally.tally();

    // 'heaven' should be the only word present (starts with 'h' but isn't 'hell')
    assert!(result.iter().any(|(word, _)| word.as_ref() == "heaven"));
    assert!(!result.iter().any(|(word, _)| word.as_ref() == "hell"));

    // All other words should be excluded
    assert_eq!(result.len(), 1);
}

#[test]
fn test_min_count_graphemes() {
    let input_text = b"e\xCC\x81"; // An `"é"` is only one char

    let filters = Filters::default().with_min_chars(2);
    let options = Options::new(
        Case::default(),
        Sort::default(),
        Serialization::default(),
        filters,
        Io::ParallelBytes,
        word_tally::Performance::default(),
    );

    let tally_map = TallyMap::from_bytes(input_text.to_vec(), &options).expect("create tally map");
    let tally = WordTally::from_tally_map(tally_map, &options);

    assert_eq!(tally.count(), 0);
}
