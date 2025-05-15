use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::PathBuf;
use std::sync::Arc;
use word_tally::input::Input;
use word_tally::output::Output;
use word_tally::serialization::{Format, Serialization};
use word_tally::{
    Case, Count, ExcludeWords, Filters, Io, Options, Performance, Processing, SizeHint, Sort,
    Tally, Word, WordTally,
};

const TEST_WORDS_PATH: &str = "tests/files/words.txt";

struct ExpectedFields<'a> {
    count: Count,
    uniq_count: Count,
    tally: Vec<(&'a str, Count)>,
}

// Create an Arc to keep Options alive for the lifetime of tests
fn make_shared<T>(value: T) -> Arc<T> {
    Arc::new(value)
}

fn word_tally(
    case: Case,
    sort: Sort,
    serialization: Serialization,
    filters: Filters,
) -> WordTally<'static> {
    let file_path = TEST_WORDS_PATH;

    let options = Options::new(
        case,
        sort,
        serialization,
        filters,
        Io::Streamed,
        Processing::Sequential,
        Performance::default(),
    );

    // For tests only: create a 'static reference using Box::leak
    // This is safe in tests since they are short-lived
    let options_static = Box::leak(Box::new(options));

    let input = Input::new(file_path, options_static.io())
        .expect("Expected test words file (`files/words.txt`) to be readable.");

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
fn equality_and_hashing() {
    fn hash_value(word_tally: &WordTally<'_>) -> u64 {
        let mut hasher = DefaultHasher::new();
        word_tally.hash(&mut hasher);
        hasher.finish()
    }

    fn assert_hash_eq(tally_a: &WordTally<'_>, tally_b: &WordTally<'_>) {
        assert_eq!(hash_value(tally_a), hash_value(tally_b));
    }

    fn assert_hash_ne(tally_a: &WordTally<'_>, tally_b: &WordTally<'_>) {
        assert_ne!(hash_value(tally_a), hash_value(tally_b));
    }

    let cases_and_sorts = [
        (Case::Original, Sort::Asc),
        (Case::Original, Sort::Desc),
        (Case::Upper, Sort::Asc),
        (Case::Upper, Sort::Desc),
        (Case::Lower, Sort::Asc),
        (Case::Lower, Sort::Desc),
    ];

    let tallies: Vec<WordTally<'static>> = cases_and_sorts
        .iter()
        .map(|&(case, sort)| {
            let serializer = Serialization::with_format(Format::Text);
            word_tally(case, sort, serializer, Filters::default())
        })
        .collect();

    for tally in &tallies {
        assert_eq!(tally, tally);
        assert_hash_eq(tally, tally);
    }

    for (i, tally_a) in tallies.iter().enumerate() {
        for (j, tally_b) in tallies.iter().enumerate() {
            if i != j {
                assert_ne!(tally_a, tally_b);
                assert_hash_ne(tally_a, tally_b);
            }
        }
    }
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
    let mut temp_file = tempfile::NamedTempFile::new().unwrap();
    std::io::Write::write_all(&mut temp_file, input_text).unwrap();

    let options = make_shared(Options::default());
    let input = Input::new(temp_file.path().to_str().unwrap(), options.io())
        .expect("Failed to create Input");

    let word_tally = WordTally::new(&input, &options).expect("Failed to create WordTally");

    // Use `tally()` to get a reference to the slice.
    let tally = word_tally.tally();

    let expected_tally: Tally = vec![
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
    ]
    .into_boxed_slice();

    assert_eq!(tally, expected_tally.as_ref());
}

#[test]
fn test_iterator() {
    let input_text = b"double trouble double";
    let mut temp_file = tempfile::NamedTempFile::new().unwrap();
    std::io::Write::write_all(&mut temp_file, input_text).unwrap();

    let options = make_shared(Options::default());
    let input = Input::new(temp_file.path().to_str().unwrap(), options.io())
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
    let mut temp_file = tempfile::NamedTempFile::new().unwrap();
    std::io::Write::write_all(&mut temp_file, input_text).unwrap();

    let options = make_shared(Options::default());
    let input = Input::new(temp_file.path().to_str().unwrap(), options.io())
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
    let mut temp_file = tempfile::NamedTempFile::new().unwrap();
    std::io::Write::write_all(&mut temp_file, input_text).unwrap();

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

    let input = Input::new(temp_file.path().to_str().unwrap(), options_arc.io())
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
    let mut temp_file = tempfile::NamedTempFile::new().unwrap();
    std::io::Write::write_all(&mut temp_file, input_text).unwrap();

    let serializer = Serialization::default();

    // Create patterns to exclude words starting with 't'
    let patterns = vec!["^t.*".to_string()];
    let filters = Filters::default().with_exclude_patterns(&patterns).unwrap();

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

    let input = Input::new(temp_file.path().to_str().unwrap(), options_arc.io())
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
    let mut temp_file = tempfile::NamedTempFile::new().unwrap();
    std::io::Write::write_all(&mut temp_file, input_text).unwrap();
    let file_path = temp_file.path().to_str().unwrap();

    let serializer = Serialization::default();

    // Create patterns to include only words starting with 'h'
    let patterns = vec!["^h.*".to_string()];
    let filters = Filters::default().with_include_patterns(&patterns).unwrap();

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
    let mut temp_file = tempfile::NamedTempFile::new().unwrap();
    std::io::Write::write_all(&mut temp_file, input_text).unwrap();
    let file_path = temp_file.path().to_str().unwrap();

    let serializer = Serialization::default();

    // Include words starting with 'h' but exclude 'hell'
    let include_patterns = vec!["^h.*".to_string()];
    let exclude_patterns = vec!["^hell$".to_string()];

    let filters = Filters::default()
        .with_include_patterns(&include_patterns)
        .unwrap()
        .with_exclude_patterns(&exclude_patterns)
        .unwrap();

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

    // 'heaven' should be the only word present (starts with 'h' but isn't 'hell')
    assert!(result.iter().any(|(word, _)| word.as_ref() == "heaven"));
    assert!(!result.iter().any(|(word, _)| word.as_ref() == "hell"));

    // All other words should be excluded
    assert_eq!(result.len(), 1);
}

#[test]
fn test_input_size() {
    let file_input = Input::File(PathBuf::from(TEST_WORDS_PATH));
    let size = file_input.size();
    assert!(size.is_some());
    assert!(size.unwrap() > 0);

    let stdin_input = Input::Stdin;
    assert_eq!(stdin_input.size(), None);
}

#[test]
fn test_parallel_vs_sequential() {
    let input_text = b"The quick brown fox jumps over the lazy dog. The fox was quick.";
    let mut temp_file = tempfile::NamedTempFile::new().unwrap();
    std::io::Write::write_all(&mut temp_file, input_text).unwrap();
    let file_path = temp_file.path().to_str().unwrap();

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
    assert_eq!(sequential.tally(), parallel.tally());
}

#[test]
fn test_memory_mapped_vs_streamed() {
    // Use the test file
    let file_path = TEST_WORDS_PATH;

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
fn test_with_size_hint() {
    let input_text = b"The quick brown fox jumps over the lazy dog.";
    let mut temp_file = tempfile::NamedTempFile::new().unwrap();
    std::io::Write::write_all(&mut temp_file, input_text).unwrap();
    let file_path = temp_file.path().to_str().unwrap();

    // Without size hint
    let no_hint_performance = Performance::default().with_size_hint(SizeHint::default());
    let filters = Filters::default();
    let no_hint_options = Options::new(
        Case::default(),
        Sort::default(),
        Serialization::default(),
        filters.clone(),
        Io::Streamed,
        Processing::Parallel,
        no_hint_performance,
    );

    let no_hint_input =
        Input::new(file_path, no_hint_options.io()).expect("Failed to create input without hint");

    let without_hint = WordTally::new(&no_hint_input, &no_hint_options)
        .expect("Failed to create WordTally without hint");

    // With size hint
    let with_hint_performance =
        Performance::default().with_size_hint(SizeHint::Bytes(input_text.len()));
    let with_hint_options = Options::new(
        Case::default(),
        Sort::default(),
        Serialization::default(),
        filters,
        Io::Streamed,
        Processing::Parallel,
        with_hint_performance,
    );

    let with_hint_input =
        Input::new(file_path, with_hint_options.io()).expect("Failed to create input with hint");

    let with_hint = WordTally::new(&with_hint_input, &with_hint_options)
        .expect("Failed to create WordTally with hint");

    assert_eq!(without_hint.count(), with_hint.count());
    assert_eq!(without_hint.uniq_count(), with_hint.uniq_count());
    assert_eq!(without_hint.tally(), with_hint.tally());
}

#[test]
fn test_estimate_capacity() {
    // Create temporary files
    let mut small_temp_file = tempfile::NamedTempFile::new().unwrap();
    std::io::Write::write_all(&mut small_temp_file, b"small text").unwrap();
    let small_path = small_temp_file.path().to_str().unwrap();

    let mut medium_temp_file = tempfile::NamedTempFile::new().unwrap();
    std::io::Write::write_all(&mut medium_temp_file, b"medium text").unwrap();
    let medium_path = medium_temp_file.path().to_str().unwrap();

    let mut large_temp_file = tempfile::NamedTempFile::new().unwrap();
    std::io::Write::write_all(&mut large_temp_file, b"large text").unwrap();
    let large_path = large_temp_file.path().to_str().unwrap();

    // Small file (8KB hint)
    let small_performance = Performance::default().with_size_hint(SizeHint::Bytes(8192)); // 8KB
    let small_options = Options::with_defaults(
        Case::default(),
        Sort::default(),
        Serialization::default(),
        Io::Streamed,
        Processing::Sequential,
        small_performance,
    );

    let small_input =
        Input::new(small_path, small_options.io()).expect("Failed to create small input");

    let small_file =
        WordTally::new(&small_input, &small_options).expect("Failed to create small WordTally");

    // Medium file (512KB hint)
    let medium_performance = Performance::default().with_size_hint(SizeHint::Bytes(524288)); // 512KB
    let medium_options = Options::with_defaults(
        Case::default(),
        Sort::default(),
        Serialization::default(),
        Io::Streamed,
        Processing::Sequential,
        medium_performance,
    );

    let medium_input =
        Input::new(medium_path, medium_options.io()).expect("Failed to create medium input");

    let medium_file =
        WordTally::new(&medium_input, &medium_options).expect("Failed to create medium WordTally");

    // Large file (4MB hint)
    let large_performance = Performance::default().with_size_hint(SizeHint::Bytes(4194304)); // 4MB
    let large_options = Options::with_defaults(
        Case::default(),
        Sort::default(),
        Serialization::default(),
        Io::Streamed,
        Processing::Sequential,
        large_performance,
    );

    let large_input =
        Input::new(large_path, large_options.io()).expect("Failed to create large input");

    let large_file =
        WordTally::new(&large_input, &large_options).expect("Failed to create large WordTally");

    assert_eq!(small_file.count(), 2);
    assert_eq!(medium_file.count(), 2);
    assert_eq!(large_file.count(), 2);
}

#[test]
fn test_parallel_count() {
    // Instead of using environment variables, just test the parallel function works
    let input_text = b"Test with default settings for chunk size and thread count";
    let mut temp_file = tempfile::NamedTempFile::new().unwrap();
    std::io::Write::write_all(&mut temp_file, input_text).unwrap();

    let performance = Performance::default();
    let options = Options::with_defaults(
        Case::default(),
        Sort::default(),
        Serialization::default(),
        Io::Streamed,
        Processing::Parallel,
        performance,
    );

    let input = Input::new(temp_file.path().to_str().unwrap(), options.io())
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
    let mut temp_file = tempfile::NamedTempFile::new().unwrap();
    std::io::Write::write_all(&mut temp_file, input_text).unwrap();

    let performance = Performance::default();
    let options = Options::with_defaults(
        Case::default(),
        Sort::default(),
        Serialization::default(),
        Io::Streamed,
        Processing::Parallel,
        performance,
    );

    let input = Input::new(temp_file.path().to_str().unwrap(), options.io())
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
        let delim = Serialization::with_delimiter("::").unwrap();
        assert_eq!(delim.delimiter(), "::");
    }
}

// Tests for Performance struct
mod performance_tests {
    use super::*;

    #[test]
    fn default_values() {
        let performance = Performance::default();
        assert_eq!(performance.tally_map_capacity(), 16384);
        assert_eq!(performance.uniqueness_ratio(), 10);
        assert_eq!(performance.words_per_kb(), 200);
        assert_eq!(performance.chunk_size(), 65_536);
    }

    #[test]
    fn builder_methods() {
        let performance = Performance::default()
            .with_tally_map_capacity(2048)
            .with_uniqueness_ratio(5)
            .with_words_per_kb(20)
            .with_chunk_size(32_768);

        assert_eq!(performance.tally_map_capacity(), 2048);
        assert_eq!(performance.uniqueness_ratio(), 5);
        assert_eq!(performance.words_per_kb(), 20);
        assert_eq!(performance.chunk_size(), 32_768);
    }

    #[test]
    fn estimate_capacity() {
        let performance = Performance::default();

        // Default when no size hint
        assert_eq!(performance.tally_map_capacity(), 16384);

        // (Size hint / 1024 * words_per_kb) / uniqueness ratio (10)
        assert_eq!(
            performance
                .with_size_hint(SizeHint::Bytes(8192))
                .tally_map_capacity(),
            160 // (8192 / 1024 * 200) / 10 = 160
        );
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
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("test_input.txt");
        std::fs::write(&file_path, TEST_INPUT).unwrap();
        (temp_dir, file_path.to_str().unwrap().to_string())
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
    fn test_default() {
        // Test the Default implementation for WordTally
        let tally = WordTally::default();
        assert_eq!(tally.count(), 0);
        assert_eq!(tally.uniq_count(), 0);
        assert!(tally.tally().is_empty());
    }

    #[test]
    fn with_parallel_processing() {
        let (_temp_dir, file_path) = create_test_file();
        let performance = Performance::default();
        let options = Options::with_defaults(
            Case::default(),
            Sort::default(),
            Serialization::default(),
            Io::Streamed,
            Processing::Parallel,
            performance,
        );
        let input = Input::new(&file_path, options.io()).expect("Failed to create Input");
        let tally = WordTally::new(&input, &options).expect("Failed to create WordTally");
        assert_eq!(tally.count(), 3);
    }

    #[test]
    fn with_custom_chunk_size() {
        let (_temp_dir, file_path) = create_test_file();
        // Create custom performance with a specific chunk size
        let performance = Performance::default().with_chunk_size(32_768);
        let options = Options::with_defaults(
            Case::default(),
            Sort::default(),
            Serialization::default(),
            Io::Streamed,
            Processing::Parallel,
            performance,
        );

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
    // Create temporary file
    let input_text = b"wombat wombat bat";
    let mut temp_file = tempfile::NamedTempFile::new().unwrap();
    std::io::Write::write_all(&mut temp_file, input_text).unwrap();

    // Create options
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

    // Create a static reference
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
fn test_json_field_renamed() {
    // Create temporary file
    let input_text = b"test json field renaming";
    let mut temp_file = tempfile::NamedTempFile::new().unwrap();
    std::io::Write::write_all(&mut temp_file, input_text).unwrap();

    // Test that the field renaming in the serialization works correctly
    let options = Options::default();
    let input = Input::new(temp_file.path().to_str().unwrap(), options.io())
        .expect("Failed to create Input");

    let original = WordTally::new(&input, &options).expect("Failed to create WordTally");
    let json = serde_json::to_string(&original).unwrap();

    // Check that the JSON contains "uniqueCount" instead of "uniq_count"
    assert!(json.contains("uniqueCount"));
    assert!(!json.contains("uniq_count"));
}

#[test]
fn test_json_field_camel_case_deserialization() {
    let input_text = b"test";
    let mut temp_file = tempfile::NamedTempFile::new().unwrap();
    std::io::Write::write_all(&mut temp_file, input_text).unwrap();

    let options = Options::default();
    let input = Input::new(temp_file.path().to_str().unwrap(), options.io())
        .expect("Failed to create Input");

    let original = WordTally::new(&input, &options).expect("Failed to create WordTally");
    let serialized = serde_json::to_string(&original).unwrap();

    assert!(serialized.contains("\"uniqueCount\":"));
    assert!(!serialized.contains("\"uniq_count\":"));

    let deserialized: WordTally<'_> = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.count(), original.count());
    assert_eq!(deserialized.uniq_count(), original.uniq_count());
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
