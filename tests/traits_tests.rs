use std::path::PathBuf;

use word_tally::{
    Buffered, Options, Threads, WordTally,
    options::{
        filters::ExcludeWords, io::Io, performance::Performance, serialization::Serialization,
    },
};

#[test]
fn test_display_implementations() {
    // Test Io Display
    assert_eq!(format!("{}", Io::ParallelStream), "parallel-stream");
    assert_eq!(format!("{}", Io::ParallelInMemory), "parallel-in-memory");
    assert_eq!(format!("{}", Io::ParallelMmap), "parallel-mmap");

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
    let display = format!("{perf}");
    assert!(display.contains("Performance {"));
    assert!(display.contains("tally_capacity"));
    assert!(display.contains("uniqueness"));
    assert!(display.contains("words/kb"));
    assert!(display.contains("chunk"));
    assert!(display.contains("stdin_size"));
    assert!(display.contains("threads"));
}

#[test]
fn test_newtype_conversions() {
    // Test ExcludeWords conversions
    let words = vec!["a".to_string(), "the".to_string()];
    let exclude_words = ExcludeWords::from(words);
    assert_eq!(exclude_words.len(), 2);
    assert_eq!(exclude_words[0].as_ref(), "a");
    assert_eq!(exclude_words[1].as_ref(), "the");
}

#[test]
fn test_thread_conversions() {
    // Test From<u16> for Threads
    let threads: Threads = 8u16.into();
    assert_eq!(threads, Threads::Count(8));
}

#[test]
fn test_ordering_traits() {
    let fmt1 = Serialization::default();
    let fmt2 = Serialization::Json;
    assert!(fmt1 < fmt2);

    // Test Performance ordering
    let perf1 = Performance::default().with_chunk_size(1000);
    let perf2 = Performance::default().with_chunk_size(2000);
    assert!(perf1 < perf2);

    // Test Options ordering
    let opt1 = Options::default().with_performance(Performance::default().with_chunk_size(1000));
    let opt2 = Options::default().with_performance(Performance::default().with_chunk_size(2000));
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
            "case": "lower",
            "sort": "desc",
            "serialization": {"text": {"field": " ", "entry": "\n"}},
            "filters": {"minChars": null, "minCount": null, "excludeWords": [], "excludePatterns": [], "includePatterns": []},
            "io": "parallelStream",
            "performance": {"uniquenessRatio": 10, "wordsPerKb": 200, "chunkSize": 65536, "baseStdinSize": 262144, "threads": "all"},
            "encoding": "unicode"
        },
        "count": 8,
        "uniqCount": 2
    }"#;

    // Deserialize directly into WordTally
    let word_tally: WordTally = serde_json::from_str(json).expect("deserialize JSON");

    // Verify the values
    assert_eq!(word_tally.count(), 8);
    assert_eq!(word_tally.uniq_count(), 2);
    assert_eq!(word_tally.len(), 2);
    assert_eq!(word_tally[0].0, "hello".into());
    assert_eq!(word_tally[0].1, 5);
    assert_eq!(word_tally[1].0, "world".into());
    assert_eq!(word_tally[1].1, 3);
}

#[test]
fn test_pathbuf_as_ref() {
    // Test the AsRef pattern that was implemented for PathBuf
    let path = PathBuf::from("/tmp/output.txt");
    let option_path = Some(path);

    // Access a reference to the inner PathBuf
    let path_ref = option_path.as_ref();
    assert!(path_ref.is_some());
    assert_eq!(
        path_ref
            .expect("get path ref")
            .to_str()
            .expect("process test"),
        "/tmp/output.txt"
    );

    // Test with None
    let none_path: Option<PathBuf> = None;
    assert!(none_path.as_ref().is_none());
}

#[test]
fn test_input_display() {
    // Create a temp file so the input can be created successfully
    let mut temp_file = tempfile::NamedTempFile::new().expect("create temp file");
    std::io::Write::write_all(&mut temp_file, b"test").expect("write test data");
    let file_path = temp_file.path();

    let file_reader = Buffered::try_from(file_path).expect("create reader");
    assert!(format!("{file_reader}").contains("tmp"));

    let stdin_reader = Buffered::stdin();
    assert_eq!(format!("{stdin_reader}"), "-");
}

#[test]
fn test_const_format_fn() {
    let options = Options::default();
    let serialization = options.serialization();
    assert!(matches!(serialization, Serialization::Text { .. }));
}
