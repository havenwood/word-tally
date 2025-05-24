use std::sync::Arc;
use word_tally::input::Input;
use word_tally::output::Output;
use word_tally::{
    Case, Count, ExcludeWords, Filters, Format, Io, Options, Performance, Processing,
    Serialization, Sort, Word, WordTally,
};

fn make_shared<T>(value: T) -> Arc<T> {
    Arc::new(value)
}

struct ExpectedFields<'a> {
    count: Count,
    uniq_count: Count,
    tally: Vec<(&'a str, Count)>,
}

fn create_test_data_file() -> tempfile::NamedTempFile {
    let mut temp_file = tempfile::NamedTempFile::new().expect("create temp file");
    // Using content that matches expected test data structure
    let content = b"d\n\
                    123 123 123 123 123\n\
                    a A *** A C D 123 123\n\
                    b b b B B B B 123\n\
                    d C d d d D D D D\n\
                    c D c c c c c c C C C C C C\n\
                    123\n";
    std::io::Write::write_all(&mut temp_file, content).expect("write test data");
    temp_file
}

fn word_tally(
    case: Case,
    sort: Sort,
    serialization: Serialization,
    filters: Filters,
) -> WordTally<'static> {
    let test_file = Box::leak(Box::new(create_test_data_file()));
    let file_path = test_file.path().to_str().expect("temp file path");

    let options = Options::new(
        case,
        sort,
        serialization,
        filters,
        Io::Streamed,
        Processing::Sequential,
        Performance::default(),
    );

    let options_static = Box::leak(Box::new(options));

    let input =
        Input::new(file_path, options_static.io()).expect("Failed to create input from test file");

    WordTally::new(&input, options_static).expect("Failed to create WordTally")
}

fn word_tally_test(case: Case, sort: Sort, filters: Filters, fields: &ExpectedFields<'_>) {
    let serialization = Serialization::with_format(Format::Text);
    let word_tally = word_tally(case, sort, serialization, filters);
    assert_eq!(word_tally.count(), fields.count);
    assert_eq!(word_tally.uniq_count(), fields.uniq_count);

    let expected_tally = fields
        .tally
        .iter()
        .map(|(word, count)| (Box::from(*word), *count))
        .collect::<Vec<_>>()
        .into_boxed_slice();

    if sort == Sort::Unsorted {
        let expected_words: std::collections::HashSet<_> = expected_tally.iter().collect();
        let actual_words: std::collections::HashSet<_> = word_tally.tally().iter().collect();
        assert_eq!(expected_words, actual_words);
    } else {
        assert_eq!(word_tally.tally(), expected_tally.as_ref());
    }
}

#[test]
fn lower_case_desc_order() {
    word_tally_test(
        Case::Lower,
        Sort::Desc,
        Filters::default(),
        &ExpectedFields {
            count: 45,
            uniq_count: 5,
            tally: vec![("c", 15), ("d", 11), ("123", 9), ("b", 7), ("a", 3)],
        },
    );
}

#[test]
fn min_char_count_at_max() {
    word_tally_test(
        Case::Lower,
        Sort::Desc,
        Filters::default().with_min_chars(3),
        &ExpectedFields {
            count: 9,
            uniq_count: 1,
            tally: vec![("123", 9)],
        },
    );
}

#[test]
fn min_char_count_above_max() {
    word_tally_test(
        Case::Lower,
        Sort::Desc,
        Filters::default().with_min_chars(4),
        &ExpectedFields {
            count: 0,
            uniq_count: 0,
            tally: vec![],
        },
    );
}

#[test]
fn min_char_count_at_min() {
    word_tally_test(
        Case::Lower,
        Sort::Desc,
        Filters::default(),
        &ExpectedFields {
            count: 45,
            uniq_count: 5,
            tally: vec![("c", 15), ("d", 11), ("123", 9), ("b", 7), ("a", 3)],
        },
    );
}

#[test]
fn min_word_count_at_max() {
    word_tally_test(
        Case::Lower,
        Sort::Desc,
        Filters::default().with_min_count(15),
        &ExpectedFields {
            count: 15,
            uniq_count: 1,
            tally: vec![("c", 15)],
        },
    );
}

#[test]
fn default_case_unsorted_order() {
    word_tally_test(
        Case::default(),
        Sort::Unsorted,
        Filters::default(),
        &ExpectedFields {
            count: 45,
            uniq_count: 5,
            tally: vec![("d", 11), ("123", 9), ("a", 3), ("c", 15), ("b", 7)],
        },
    );
}

#[test]
fn upper_case_desc_order() {
    word_tally_test(
        Case::Upper,
        Sort::Desc,
        Filters::default(),
        &ExpectedFields {
            count: 45,
            uniq_count: 5,
            tally: vec![("C", 15), ("D", 11), ("123", 9), ("B", 7), ("A", 3)],
        },
    );
}

#[test]
fn lower_case_asc_order() {
    word_tally_test(
        Case::Lower,
        Sort::Asc,
        Filters::default(),
        &ExpectedFields {
            count: 45,
            uniq_count: 5,
            tally: vec![("a", 3), ("b", 7), ("123", 9), ("d", 11), ("c", 15)],
        },
    );
}

#[test]
fn upper_case_asc_order() {
    word_tally_test(
        Case::Upper,
        Sort::Asc,
        Filters::default(),
        &ExpectedFields {
            count: 45,
            uniq_count: 5,
            tally: vec![("A", 3), ("B", 7), ("123", 9), ("D", 11), ("C", 15)],
        },
    );
}

#[test]
fn original_case_desc_order() {
    word_tally_test(
        Case::Original,
        Sort::Desc,
        Filters::default(),
        &ExpectedFields {
            count: 45,
            uniq_count: 9,
            tally: vec![
                ("123", 9),
                ("C", 8),
                ("c", 7),
                ("D", 6),
                ("d", 5),
                ("B", 4),
                ("b", 3),
                ("A", 2),
                ("a", 1),
            ],
        },
    );
}

#[test]
fn original_case_asc_order() {
    word_tally_test(
        Case::Original,
        Sort::Asc,
        Filters::default(),
        &ExpectedFields {
            count: 45,
            uniq_count: 9,
            tally: vec![
                ("a", 1),
                ("A", 2),
                ("b", 3),
                ("B", 4),
                ("d", 5),
                ("D", 6),
                ("c", 7),
                ("C", 8),
                ("123", 9),
            ],
        },
    );
}

#[test]
fn vec_from() {
    let tally = word_tally(
        Case::default(),
        Sort::default(),
        Serialization::default(),
        Filters::default(),
    );

    assert_eq!(
        Vec::from(tally),
        vec![
            (Box::from("c"), 15),
            (Box::from("d"), 11),
            (Box::from("123"), 9),
            (Box::from("b"), 7),
            (Box::from("a"), 3)
        ]
    );
}

#[test]
fn test_into_tally() {
    let input_text = b"Hope is the thing with feathers that perches in the soul";
    let mut temp_file = tempfile::NamedTempFile::new().expect("create temp file");
    std::io::Write::write_all(&mut temp_file, input_text).expect("write test data");

    let options = make_shared(Options::default());
    let input = Input::new(
        temp_file.path().to_str().expect("temp file path"),
        options.io(),
    )
    .expect("Failed to create Input");

    let word_tally = WordTally::new(&input, &options).expect("Failed to create WordTally");

    // Use `tally()` to get a reference to the slice.
    let tally = word_tally.tally();

    let mut tally_vec: Vec<_> = tally.to_vec();
    tally_vec.sort_by_key(|(word, _): &(Word, Count)| word.clone());

    let mut expected_tally = vec![
        ("the".into(), 2),
        ("hope".into(), 1),
        ("is".into(), 1),
        ("thing".into(), 1),
        ("with".into(), 1),
        ("feathers".into(), 1),
        ("that".into(), 1),
        ("perches".into(), 1),
        ("in".into(), 1),
        ("soul".into(), 1),
    ];
    expected_tally.sort_by_key(|(word, _): &(Word, Count)| word.clone());

    assert_eq!(tally_vec, expected_tally);
}

#[test]
fn test_iterator() {
    let input_text = b"double trouble double";
    let mut temp_file = tempfile::NamedTempFile::new().expect("create temp file");
    std::io::Write::write_all(&mut temp_file, input_text).expect("write test data");

    let options = make_shared(Options::default());
    let input = Input::new(
        temp_file.path().to_str().expect("temp file path"),
        options.io(),
    )
    .expect("Failed to create Input");

    let word_tally = WordTally::new(&input, &options).expect("Failed to create WordTally");

    let expected: Vec<(Word, Count)> = vec![(Box::from("double"), 2), (Box::from("trouble"), 1)];

    let collected: Vec<(Word, Count)> = (&word_tally).into_iter().cloned().collect();
    assert_eq!(collected, expected);

    let mut iter = (&word_tally).into_iter();
    assert_eq!(iter.next(), Some(&(Box::from("double"), 2)));
    assert_eq!(iter.next(), Some(&(Box::from("trouble"), 1)));
    assert_eq!(iter.next(), None);
}

#[test]
fn test_iterator_for_loop() {
    let input_text = b"llama llama pajamas";
    let mut temp_file = tempfile::NamedTempFile::new().expect("create temp file");
    std::io::Write::write_all(&mut temp_file, input_text).expect("write test data");

    let options = make_shared(Options::default());
    let input = Input::new(
        temp_file.path().to_str().expect("temp file path"),
        options.io(),
    )
    .expect("Failed to create Input");

    let word_tally = WordTally::new(&input, &options).expect("Failed to create WordTally");

    let expected: Vec<(Word, Count)> = vec![(Box::from("llama"), 2), (Box::from("pajamas"), 1)];

    let mut collected = vec![];
    for item in &word_tally {
        collected.push(item.clone());
    }
    assert_eq!(collected, expected);
}

#[test]
fn test_excluding_words() {
    let input_text = "The tree that would grow to heaven must send its roots to hell.".as_bytes();
    let mut temp_file = tempfile::NamedTempFile::new().expect("create temp file");
    std::io::Write::write_all(&mut temp_file, input_text).expect("write test data");

    let words = vec!["Heaven".to_string(), "Hell".to_string()];
    let serializer = Serialization::default();
    let filters = Filters::default().with_exclude_words(words);
    let options = Options::new(
        Case::default(),
        Sort::Unsorted,
        serializer,
        filters,
        Io::Streamed,
        Processing::Sequential,
        Performance::default(),
    );
    let options_arc = make_shared(options);

    let input = Input::new(
        temp_file.path().to_str().expect("temp file path"),
        options_arc.io(),
    )
    .expect("Failed to create Input");

    let tally = WordTally::new(&input, &options_arc).expect("Failed to create WordTally");
    let result = tally.tally();

    assert!(result.iter().any(|(word, _)| word.as_ref() == "tree"));
    assert!(!result.iter().any(|(word, _)| word.as_ref() == "heaven"));
    assert!(!result.iter().any(|(word, _)| word.as_ref() == "hell"));
}

#[test]
fn test_excluding_patterns() {
    let input_text = "The tree that would grow to heaven must send its roots to hell.".as_bytes();
    let mut temp_file = tempfile::NamedTempFile::new().expect("create temp file");
    std::io::Write::write_all(&mut temp_file, input_text).expect("write test data");

    let serializer = Serialization::default();

    // Exclude words starting with 't'
    let patterns = vec!["^t.*".to_string()];
    let filters = Filters::default()
        .with_exclude_patterns(&patterns)
        .expect("set exclude patterns");

    let options = Options::new(
        Case::default(),
        Sort::Unsorted,
        serializer,
        filters,
        Io::Streamed,
        Processing::Sequential,
        Performance::default(),
    );
    let options_arc = make_shared(options);

    let input = Input::new(
        temp_file.path().to_str().expect("temp file path"),
        options_arc.io(),
    )
    .expect("Failed to create Input");

    let tally = WordTally::new(&input, &options_arc).expect("Failed to create WordTally");
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
    std::io::Write::write_all(&mut temp_file, input_text).expect("write test data");
    let file_path = temp_file.path().to_str().expect("temp file path");

    let serializer = Serialization::default();

    // Include only words starting with `'h'`
    let patterns = vec!["^h.*".to_string()];
    let filters = Filters::default()
        .with_include_patterns(&patterns)
        .expect("set include patterns");

    let options = Options::new(
        Case::default(),
        Sort::Unsorted,
        serializer,
        filters,
        Io::Streamed,
        Processing::Sequential,
        Performance::default(),
    );
    let options_arc = make_shared(options);

    let input = Input::new(file_path, options_arc.io()).expect("Failed to create Input");

    let tally = WordTally::new(&input, &options_arc).expect("Failed to create WordTally");
    let result = tally.tally();

    // These should be present (words starting with `'h'`)
    assert!(result.iter().any(|(word, _)| word.as_ref() == "heaven"));
    assert!(result.iter().any(|(word, _)| word.as_ref() == "hell"));

    // These should be excluded (words not starting with `'h'`)
    assert!(!result.iter().any(|(word, _)| word.as_ref() == "tree"));
    assert!(!result.iter().any(|(word, _)| word.as_ref() == "would"));
    assert!(!result.iter().any(|(word, _)| word.as_ref() == "to"));
    assert!(!result.iter().any(|(word, _)| word.as_ref() == "the"));

    // Make sure we only have words starting with `'h'`
    assert!(result.iter().all(|(word, _)| word.starts_with('h')));
}

#[test]
fn test_combining_include_exclude_patterns() {
    let input_text = "The tree that would grow to heaven must send its roots to hell.".as_bytes();
    let mut temp_file = tempfile::NamedTempFile::new().expect("create temp file");
    std::io::Write::write_all(&mut temp_file, input_text).expect("write test data");
    let file_path = temp_file.path().to_str().expect("temp file path");

    let serializer = Serialization::default();

    // Include words starting with `'h'` but exclude `'hell'`
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
        Io::Streamed,
        Processing::Sequential,
        Performance::default(),
    );
    let options_arc = make_shared(options);

    let input = Input::new(file_path, options_arc.io()).expect("Failed to create Input");

    let tally = WordTally::new(&input, &options_arc).expect("Failed to create WordTally");
    let result = tally.tally();

    // `'heaven'` should be the only word present (starts with `'h'` but isn't `'hell'`)
    assert!(result.iter().any(|(word, _)| word.as_ref() == "heaven"));
    assert!(!result.iter().any(|(word, _)| word.as_ref() == "hell"));

    // All other words should be excluded
    assert_eq!(result.len(), 1);
}

#[test]
fn test_input_size() {
    let temp_file = create_test_data_file();
    let file_input = Input::File(temp_file.path().to_path_buf());
    let size = file_input.size();
    assert!(size.is_some());
    assert!(size.expect("get file size") > 0);

    let stdin_input = Input::Stdin;
    assert_eq!(stdin_input.size(), None);
}

#[test]
fn test_parallel_vs_sequential() {
    let input_text = b"I taste a liquor never brewed. I taste a liquor.";
    let mut temp_file = tempfile::NamedTempFile::new().expect("create temp file");
    std::io::Write::write_all(&mut temp_file, input_text).expect("write test data");
    let file_path = temp_file.path().to_str().expect("temp file path");

    // Sequential processing
    let seq_performance = Performance::default();
    let filters = Filters::default();
    let seq_options = Options::new(
        Case::default(),
        Sort::default(),
        Serialization::default(),
        filters.clone(),
        Io::Streamed,
        Processing::Sequential,
        seq_performance,
    );
    let seq_options_arc = make_shared(seq_options);

    let seq_input =
        Input::new(file_path, seq_options_arc.io()).expect("Failed to create sequential input");

    let sequential = WordTally::new(&seq_input, &seq_options_arc)
        .expect("Failed to create sequential WordTally");

    // Parallel processing
    let par_performance = Performance::default();
    let par_options = Options::new(
        Case::default(),
        Sort::default(),
        Serialization::default(),
        filters,
        Io::Streamed,
        Processing::Parallel,
        par_performance,
    );
    let par_options_arc = make_shared(par_options);

    let par_input =
        Input::new(file_path, par_options_arc.io()).expect("Failed to create parallel input");

    let parallel =
        WordTally::new(&par_input, &par_options_arc).expect("Failed to create parallel WordTally");

    assert_eq!(sequential.count(), parallel.count());
    assert_eq!(sequential.uniq_count(), parallel.uniq_count());

    let mut seq_tally: Vec<_> = sequential.tally().to_vec();
    seq_tally.sort_by_key(|(word, _): &(Word, Count)| word.clone());

    let mut par_tally: Vec<_> = parallel.tally().to_vec();
    par_tally.sort_by_key(|(word, _): &(Word, Count)| word.clone());

    assert_eq!(seq_tally, par_tally);
}

#[test]
fn test_memory_mapped_vs_streamed() {
    // Use the test file
    let test_file = create_test_data_file();
    let file_path = test_file.path().to_str().expect("temp file path");

    // Set up options for memory-mapped I/O (sequential)
    let mmap_performance = Performance::default();
    let filters = Filters::default();
    let mmap_options = Options::new(
        Case::default(),
        Sort::default(),
        Serialization::default(),
        filters.clone(),
        Io::MemoryMapped,
        Processing::Sequential,
        mmap_performance,
    );

    // Set up options for streaming I/O (sequential)
    let stream_performance = Performance::default();
    let stream_options = Options::new(
        Case::default(),
        Sort::default(),
        Serialization::default(),
        filters.clone(),
        Io::Streamed,
        Processing::Sequential,
        stream_performance,
    );

    // Create inputs with the different I/O modes
    let mmap_input =
        Input::new(file_path, mmap_options.io()).expect("Failed to create memory-mapped input");
    let stream_input =
        Input::new(file_path, stream_options.io()).expect("Failed to create streamed input");

    // Create WordTally instances with the different I/O modes
    let memory_mapped = WordTally::new(&mmap_input, &mmap_options).expect("Memory mapping failed");
    let streamed = WordTally::new(&stream_input, &stream_options)
        .expect("Failed to create streamed WordTally");

    // Verify results are the same regardless of I/O mode
    assert_eq!(memory_mapped.count(), streamed.count());
    assert_eq!(memory_mapped.uniq_count(), streamed.uniq_count());
    assert_eq!(memory_mapped.tally(), streamed.tally());

    // Now test with parallel processing
    // Set up options for parallel streamed I/O
    let parallel_performance = Performance::default();
    let parallel_options = Options::new(
        Case::default(),
        Sort::default(),
        Serialization::default(),
        filters,
        Io::Streamed,
        Processing::Parallel,
        parallel_performance,
    );

    // Create input for parallel streamed processing
    let parallel_input =
        Input::new(file_path, parallel_options.io()).expect("Failed to create parallel input");

    // Create WordTally instance with parallel streamed processing
    let parallel_stream = WordTally::new(&parallel_input, &parallel_options)
        .expect("Failed to create parallel stream WordTally");

    // Verify the parallel processing worked
    assert!(parallel_stream.count() > 0);
    assert!(parallel_stream.uniq_count() > 0);
}

#[test]
fn test_parallel_count() {
    // Instead of using environment variables, just test the parallel function works
    let input_text = b"Test with default settings for chunk size and thread count";
    let mut temp_file = tempfile::NamedTempFile::new().expect("create temp file");
    std::io::Write::write_all(&mut temp_file, input_text).expect("write test data");

    let options = Options::default().with_processing(Processing::Parallel);

    let input = Input::new(
        temp_file.path().to_str().expect("temp file path"),
        options.io(),
    )
    .expect("Failed to create Input");

    let parallel = WordTally::new(&input, &options).expect("Failed to create parallel WordTally");

    // Only check the counts are positive numbers (actual counts may vary by implementation)
    assert!(parallel.count() > 0);
    assert!(parallel.uniq_count() > 0);
    // Also check uniq count is less than or equal to total count
    assert!(parallel.uniq_count() <= parallel.count());
}

#[test]
fn test_merge_maps() {
    let input_text = b"This is a test of the map merging functionality";
    let mut temp_file = tempfile::NamedTempFile::new().expect("create temp file");
    std::io::Write::write_all(&mut temp_file, input_text).expect("write test data");

    let options = Options::default().with_processing(Processing::Parallel);

    let input = Input::new(
        temp_file.path().to_str().expect("temp file path"),
        options.io(),
    )
    .expect("Failed to create Input");

    let tally = WordTally::new(&input, &options).expect("Failed to create WordTally");

    assert_eq!(tally.count(), 9);
    assert_eq!(tally.uniq_count(), 9);
}

#[test]
fn test_words_exclude_from() {
    let words = vec!["beep".to_string(), "boop".to_string()];
    assert_eq!(ExcludeWords::from(words.clone()), ExcludeWords(words));
}

// Tests for Serialization convenience methods
mod serialization_tests {
    use super::*;

    #[test]
    fn with_format() {
        let format_only = Serialization::with_format(Format::Json);
        assert_eq!(format_only.format(), Format::Json);
    }

    #[test]
    fn with_delimiter() {
        let delim = Serialization::with_delimiter("::").expect("create delimiter");
        assert_eq!(delim.delimiter(), "::");
    }
}

// Tests for Default implementations
mod default_impl_tests {
    use super::*;

    #[test]
    fn input_default() {
        let input = Input::default();
        assert!(matches!(input, Input::Stdin));
    }

    #[test]
    fn output_default() {
        let _output = Output::default();
        // Just verify it compiles
    }
}

// Tests for WordTally convenience constructors
mod wordtally_constructor_tests {
    use super::*;

    const TEST_INPUT: &[u8] = b"test convenience constructors";

    fn create_test_file() -> (tempfile::TempDir, String) {
        let temp_dir = tempfile::tempdir().expect("process test");
        let file_path = temp_dir.path().join("test_input.txt");
        std::fs::write(&file_path, TEST_INPUT).expect("process test");
        (
            temp_dir,
            file_path.to_str().expect("process test").to_string(),
        )
        // temp_dir will be kept alive until it's dropped
    }

    #[test]
    fn with_defaults() {
        let (_temp_dir, file_path) = create_test_file();
        let options = Options::default();
        let input = Input::new(&file_path, options.io()).expect("Failed to create Input");
        let tally = WordTally::new(&input, &options).expect("Failed to create WordTally");
        assert_eq!(tally.count(), 3);
    }

    #[test]
    fn with_parallel_processing() {
        let (_temp_dir, file_path) = create_test_file();
        let options = Options::default().with_processing(Processing::Parallel);
        let input = Input::new(&file_path, options.io()).expect("Failed to create Input");
        let tally = WordTally::new(&input, &options).expect("Failed to create WordTally");
        assert_eq!(tally.count(), 3);
    }

    #[test]
    fn with_custom_chunk_size() {
        let (_temp_dir, file_path) = create_test_file();
        let options = Options::default()
            .with_processing(Processing::Parallel)
            .with_chunk_size(32_768);

        let input = Input::new(&file_path, options.io()).expect("Failed to create Input");
        let tally = WordTally::new(&input, &options).expect("Failed to create WordTally");
        assert_eq!(tally.count(), 3);
    }
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
        Io::Streamed,
        Processing::Sequential,
        Performance::default(),
    );

    let input = Input::from_bytes(input_text);
    let tally = WordTally::new(&input, &options).expect("Failed to create WordTally");

    assert_eq!(tally.count(), 0);
}

#[test]
fn test_to_json() {
    let input_text = b"wombat wombat bat";
    let mut temp_file = tempfile::NamedTempFile::new().expect("create temp file");
    std::io::Write::write_all(&mut temp_file, input_text).expect("write test data");
    let filters = Filters::default();
    let options = Options::new(
        Case::default(),
        Sort::default(),
        Serialization::default(),
        filters,
        Io::Streamed,
        Processing::Sequential,
        Performance::default(),
    );

    let shared_options = make_shared(options);

    let input = Input::new(
        temp_file.path().to_str().expect("temp file path"),
        shared_options.io(),
    )
    .expect("Failed to create Input");

    let expected = WordTally::new(&input, &shared_options).expect("Failed to create WordTally");
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

    let filters = Filters::default();
    let options = Options::new(
        Case::default(),
        Sort::default(),
        Serialization::default(),
        filters,
        Io::Streamed,
        Processing::Sequential,
        Performance::default(),
    );

    let input = Input::new(
        temp_file.path().to_str().expect("temp file path"),
        options.io(),
    )
    .expect("Failed to create Input");

    let original = WordTally::new(&input, &options).expect("Failed to create WordTally");
    let json = serde_json::to_string(&original).expect("serialize JSON");
    let deserialized: WordTally<'_> = serde_json::from_str(&json).expect("deserialize JSON");

    assert_eq!(deserialized.count(), original.count());
    assert_eq!(deserialized.uniq_count(), original.uniq_count());
    assert_eq!(deserialized.tally(), original.tally());
    assert_eq!(deserialized.options().case(), original.options().case());
    assert_eq!(deserialized.options().sort(), original.options().sort());
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
    .expect("Failed to create Input");

    let original = WordTally::new(&input, &options).expect("Failed to create WordTally");
    let json = serde_json::to_string(&original).expect("serialize JSON");
    let deserialized: WordTally<'_> = serde_json::from_str(&json).expect("deserialize JSON");

    assert_eq!(deserialized.count(), original.count());
    assert_eq!(deserialized.uniq_count(), original.uniq_count());
    assert_eq!(deserialized.tally(), original.tally());

    // Options should be functionally equivalent
    assert_eq!(deserialized.options().case(), original.options().case());
    assert_eq!(deserialized.options().sort(), original.options().sort());
    // Deserialized instance will have owned options, not shared references
}

#[test]
fn test_json_field_renamed() {
    let input_text = b"test json field renaming";
    let mut temp_file = tempfile::NamedTempFile::new().expect("create temp file");
    std::io::Write::write_all(&mut temp_file, input_text).expect("write test data");

    // Test that the field renaming in the serialization works correctly
    let options = Options::default();
    let input = Input::new(
        temp_file.path().to_str().expect("temp file path"),
        options.io(),
    )
    .expect("Failed to create Input");

    let original = WordTally::new(&input, &options).expect("Failed to create WordTally");
    let json = serde_json::to_string(&original).expect("serialize JSON");

    // Check that the JSON contains `"uniqueCount"` instead of `"uniq_count"`
    assert!(json.contains("uniqueCount"));
    assert!(!json.contains("uniq_count"));
}

#[test]
fn test_json_field_camel_case_deserialization() {
    let input_text = b"test";
    let mut temp_file = tempfile::NamedTempFile::new().expect("create temp file");
    std::io::Write::write_all(&mut temp_file, input_text).expect("write test data");

    let options = Options::default();
    let input = Input::new(
        temp_file.path().to_str().expect("temp file path"),
        options.io(),
    )
    .expect("Failed to create Input");

    let original = WordTally::new(&input, &options).expect("Failed to create WordTally");
    let serialized = serde_json::to_string(&original).expect("serialize JSON");

    assert!(serialized.contains("\"uniqueCount\":"));
    assert!(!serialized.contains("\"uniq_count\":"));

    let deserialized: WordTally<'_> = serde_json::from_str(&serialized).expect("deserialize JSON");
    assert_eq!(deserialized.count(), original.count());
    assert_eq!(deserialized.uniq_count(), original.uniq_count());
}

#[test]
fn test_into_owned_converts_borrowed_to_owned() {
    let content = b"apple banana cherry";
    let mut temp_file = tempfile::NamedTempFile::new().expect("create temp file");
    std::io::Write::write_all(&mut temp_file, content).expect("write test data");

    let options = Options::default();
    let input = Input::new(temp_file.path(), options.io()).expect("process test");
    let word_tally = WordTally::new(&input, &options).expect("create word tally");

    // Store original values before consuming
    let original_count = word_tally.count();
    let original_uniq_count = word_tally.uniq_count();
    let original_tally_len = word_tally.tally().len();
    let original_case = word_tally.options().case();

    // Convert to owned
    let owned_tally = word_tally.into_owned();

    // Verify that all data is preserved
    assert_eq!(owned_tally.count(), original_count);
    assert_eq!(owned_tally.uniq_count(), original_uniq_count);
    assert_eq!(owned_tally.tally().len(), original_tally_len);
    assert_eq!(owned_tally.options().case(), original_case);
}

#[test]
fn test_into_owned_preserves_all_data() {
    let content = b"one two three one two one";
    let mut temp_file = tempfile::NamedTempFile::new().expect("create temp file");
    std::io::Write::write_all(&mut temp_file, content).expect("write test data");

    let options = Options::default()
        .with_case(Case::Upper)
        .with_sort(Sort::Desc)
        .with_filters(
            Filters::default()
                .with_min_chars(2)
                .with_exclude_words(vec!["ONE".to_string()]),
        );

    let input = Input::new(temp_file.path(), options.io()).expect("process test");
    let original = WordTally::new(&input, &options).expect("create word tally");

    // Store original values
    let original_count = original.count();
    let original_uniq_count = original.uniq_count();
    let original_tally: Vec<_> = original.tally().to_vec();
    let original_case = original.options().case();
    let original_sort = original.options().sort();
    let original_min_chars = original.options().filters().min_chars();

    // Convert to owned
    let owned = original.into_owned();

    // Verify all data is preserved
    assert_eq!(owned.count(), original_count);
    assert_eq!(owned.uniq_count(), original_uniq_count);
    assert_eq!(owned.tally(), &original_tally[..]);
    assert_eq!(owned.options().case(), original_case);
    assert_eq!(owned.options().sort(), original_sort);
    assert_eq!(owned.options().filters().min_chars(), original_min_chars);
}

#[test]
fn test_into_owned_multiple_conversions() {
    // Test that we can convert to owned multiple times
    let content = b"test multiple conversions";
    let mut temp_file = tempfile::NamedTempFile::new().expect("create temp file");
    std::io::Write::write_all(&mut temp_file, content).expect("write test data");

    let options = Options::default();
    let input = Input::new(temp_file.path(), options.io()).expect("process test");
    let word_tally = WordTally::new(&input, &options).expect("create word tally");

    // First conversion
    let owned1 = word_tally.into_owned();
    assert_eq!(owned1.count(), 3);

    // Second conversion (already owned, but should still work)
    let owned2 = owned1.into_owned();
    assert_eq!(owned2.count(), 3);
    assert_eq!(owned2.uniq_count(), 3);
}

#[test]
fn test_into_owned_with_custom_options() {
    let content = b"HELLO WORLD HELLO RUST WORLD HELLO";
    let mut temp_file = tempfile::NamedTempFile::new().expect("create temp file");
    std::io::Write::write_all(&mut temp_file, content).expect("write test data");

    let options = Options::default()
        .with_case(Case::Lower)
        .with_sort(Sort::Asc)
        .with_filters(Filters::default().with_min_count(2));

    let input = Input::new(temp_file.path(), options.io()).expect("process test");
    let original = WordTally::new(&input, &options).expect("create word tally");

    // Verify original state (min_count filter applies)
    // Only `"world"` (2) and `"hello"` (3) remain; `"rust"` (1) is filtered out
    assert_eq!(original.count(), 5); // Total words after filtering
    assert_eq!(original.uniq_count(), 2); // Unique words after filtering

    // Convert to owned
    let owned = original.into_owned();

    // Verify owned version maintains the same state
    assert_eq!(owned.count(), 5);
    assert_eq!(owned.uniq_count(), 2);
    assert_eq!(owned.options().case(), Case::Lower);
    assert_eq!(owned.options().sort(), Sort::Asc);

    // The tally should be sorted ascending by count and filtered by min_count
    let tally = owned.tally();
    assert_eq!(tally.len(), 2); // Filtered by min_count
}
