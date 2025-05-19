use std::path::PathBuf;
use word_tally::Threads;
use word_tally::options::{
    filters::ExcludeWords,
    io::Io,
    performance::Performance,
    processing::Processing,
    serialization::{Format, Serialization},
};
use word_tally::{Input, Options, WordTally};

#[test]
fn test_display_implementations() {
    // Test Io Display
    assert_eq!(format!("{}", Io::Streamed), "streamed");
    assert_eq!(format!("{}", Io::Buffered), "buffered");
    assert_eq!(format!("{}", Io::MemoryMapped), "memory-mapped");

    // Test Processing Display
    assert_eq!(format!("{}", Processing::Sequential), "sequential");
    assert_eq!(format!("{}", Processing::Parallel), "parallel");

    // Test Threads Display
    // For Threads::All, expect the actual number of threads from Rayon
    let all_threads_str = format!("{}", Threads::All);
    assert!(
        all_threads_str.parse::<usize>().is_ok(),
        "Threads::All should display as a number"
    );
    assert_eq!(format!("{}", Threads::Count(4)), "4");

    // Test Performance Display
    let perf = Performance::default();
    let display = format!("{}", perf);
    assert!(display.contains("Performance {"));
    assert!(display.contains("tally_capacity"));
    assert!(display.contains("uniqueness"));
    assert!(display.contains("words/kb"));
    assert!(display.contains("chunk"));
    assert!(display.contains("stdin_size"));
    assert!(display.contains("threads"));
    assert!(display.contains("verbose"));
}

#[test]
fn test_newtype_conversions() {
    // Test ExcludeWords conversions
    let words = vec!["a".to_string(), "the".to_string()];
    let exclude_words = ExcludeWords(words.clone());
    assert_eq!(*exclude_words.as_ref(), words);
    // Test Deref implementation
    assert_eq!(exclude_words.len(), 2);
    assert_eq!(exclude_words[0], "a");
}

#[test]
fn test_thread_conversions() {
    // Test From<u16> for Threads
    let threads: Threads = 8u16.into();
    assert_eq!(threads, Threads::Count(8));
}

#[test]
fn test_ordering_traits() {
    let fmt1 = Serialization::with_format(Format::Text);
    let fmt2 = Serialization::with_format(Format::Json);
    assert!(fmt1 < fmt2);

    // Test Performance ordering
    let perf1 = Performance::default().with_chunk_size(1000);
    let perf2 = Performance::default().with_chunk_size(2000);
    assert!(perf1 < perf2);

    // Test Options ordering
    let opt1 = Options::default().with_chunk_size(1000);
    let opt2 = Options::default().with_chunk_size(2000);
    assert!(opt1 < opt2);
}

#[test]
fn test_pattern_ordering() {
    let filters1 = word_tally::Filters::default();
    let filters2 = word_tally::Filters::default().with_min_chars(5);
    assert!(filters1 < filters2);
}

#[test]
fn test_wordtally_deserialize() {
    // Create a simple JSON representation of a WordTally
    let json = r#"{
        "tally": [["hello", 5], ["world", 3]],
        "options": {
            "case": "Lower",
            "sort": "Desc",
            "serialization": {"format": "Text", "delimiter": " "},
            "filters": {"min_chars": null, "min_count": null, "exclude_words": [], "exclude_patterns": [], "include_patterns": []},
            "io": "Streamed",
            "processing": "Sequential",
            "performance": {"base_stdin_tally_capacity": 5120, "uniqueness_ratio": 10, "words_per_kb": 200, "chunk_size": 65536, "base_stdin_size": 262144, "threads": "All", "verbose": false}
        },
        "count": 8,
        "uniqueCount": 2
    }"#;

    // Deserialize directly into WordTally
    let word_tally: WordTally<'_> = serde_json::from_str(json).unwrap();

    // Verify the values
    assert_eq!(word_tally.count(), 8);
    assert_eq!(word_tally.uniq_count(), 2);
    assert_eq!(word_tally.tally().len(), 2);
    assert_eq!(word_tally.tally()[0].0, "hello".into());
    assert_eq!(word_tally.tally()[0].1, 5);
    assert_eq!(word_tally.tally()[1].0, "world".into());
    assert_eq!(word_tally.tally()[1].1, 3);
}

#[test]
fn test_pathbuf_as_ref() {
    // Test the AsRef pattern that was implemented for PathBuf
    let path = PathBuf::from("/tmp/output.txt");
    let option_path = Some(path);

    // Access a reference to the inner PathBuf
    let path_ref = option_path.as_ref();
    assert!(path_ref.is_some());
    assert_eq!(path_ref.unwrap().to_str().unwrap(), "/tmp/output.txt");

    // Test with None
    let none_path: Option<PathBuf> = None;
    assert!(none_path.as_ref().is_none());
}

#[test]
fn test_input_display() {
    let file_input = Input::File(PathBuf::from("/tmp/test.txt"));
    assert_eq!(format!("{}", file_input), "File(/tmp/test.txt)");

    let stdin_input = Input::Stdin;
    assert_eq!(format!("{}", stdin_input), "Stdin");
}

#[test]
fn test_const_format_fn() {
    let options = Options::default();
    let format = options.serialization().format();
    assert_eq!(format, Format::Text);
}
