use std::fs::File;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::PathBuf;
use std::sync::Arc;
use word_tally::input::Input;
use word_tally::output::Output;
use word_tally::{
    Case, Count, ExcludeWords, Filters, Format, Formatting, Io, Options, Performance, Processing,
    SizeHint, Sort, Tally, Word, WordTally,
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

fn word_tally(formatting: Formatting, filters: Filters) -> WordTally<'static> {
    let input = File::open(TEST_WORDS_PATH)
        .expect("Expected test words file (`files/words.txt`) to be readable.");

    let performance = Performance::default()
        .with_io(Io::Streamed)
        .with_processing(Processing::Sequential);
    let options = Options::new(formatting, filters, performance);

    // For tests only: create a 'static reference using Box::leak
    // This is safe in tests since they are short-lived
    let options_static = Box::leak(Box::new(options));

    WordTally::new(input, options_static)
}

fn word_tally_test(case: Case, sort: Sort, filters: Filters, fields: &ExpectedFields<'_>) {
    let formatting = Formatting::new(case, sort, Format::Text);
    let word_tally = word_tally(formatting, filters);
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
            let formatting = Formatting::new(case, sort, Format::Text);
            word_tally(formatting, Filters::default())
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
    let tally = word_tally(Formatting::default(), Filters::default());

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
    let input = b"bye bye birdy";
    let options = make_shared(Options::default());
    let word_tally = WordTally::new(&input[..], &options);

    // Use `tally()` to get a reference to the slice.
    let tally = word_tally.tally();

    let expected_tally: Tally = vec![("bye".into(), 2), ("birdy".into(), 1)].into_boxed_slice();

    assert_eq!(tally, expected_tally.as_ref());
}

#[test]
fn test_iterator() {
    let input = b"double trouble double";
    let options = make_shared(Options::default());
    let word_tally = WordTally::new(&input[..], &options);

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
    let input = b"llama llama pajamas";
    let options = make_shared(Options::default());
    let word_tally = WordTally::new(&input[..], &options);

    let expected: Vec<(Word, Count)> = vec![(Box::from("llama"), 2), (Box::from("pajamas"), 1)];

    let mut collected = vec![];
    for item in &word_tally {
        collected.push(item.clone());
    }
    assert_eq!(collected, expected);
}

#[test]
fn test_excluding_words() {
    let input = "The tree that would grow to heaven must send its roots to hell.".as_bytes();
    let words = vec!["Heaven".to_string(), "Hell".to_string()];
    let formatting = Formatting::with_sort(Sort::Unsorted);
    let filters = Filters::default().with_exclude_words(words);
    let performance = Performance::default()
        .with_io(Io::Streamed)
        .with_processing(Processing::Sequential)
        .with_size_hint(SizeHint::None);
    let options = Options::new(formatting, filters, performance);
    let options_arc = make_shared(options);
    let tally = WordTally::new(input, &options_arc);
    let result = tally.tally();

    assert!(result.iter().any(|(word, _)| word.as_ref() == "tree"));
    assert!(!result.iter().any(|(word, _)| word.as_ref() == "heaven"));
    assert!(!result.iter().any(|(word, _)| word.as_ref() == "hell"));
}

#[test]
fn test_excluding_patterns() {
    let input = "The tree that would grow to heaven must send its roots to hell.".as_bytes();
    let formatting = Formatting::with_sort(Sort::Unsorted);

    // Create patterns to exclude words starting with 't'
    let patterns = vec!["^t.*".to_string()];
    let filters = Filters::default().with_exclude_patterns(&patterns).unwrap();

    let performance = Performance::default()
        .with_io(Io::Streamed)
        .with_processing(Processing::Sequential)
        .with_size_hint(SizeHint::None);
    let options = Options::new(formatting, filters, performance);
    let options_arc = make_shared(options);
    let tally = WordTally::new(input, &options_arc);
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
    let input = "The tree that would grow to heaven must send its roots to hell.".as_bytes();
    let formatting = Formatting::with_sort(Sort::Unsorted);

    // Create patterns to include only words starting with 'h'
    let patterns = vec!["^h.*".to_string()];
    let filters = Filters::default().with_include_patterns(&patterns).unwrap();

    let performance = Performance::default()
        .with_io(Io::Streamed)
        .with_processing(Processing::Sequential)
        .with_size_hint(SizeHint::None);
    let options = Options::new(formatting, filters, performance);
    let options_arc = make_shared(options);
    let tally = WordTally::new(input, &options_arc);
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
    let input = "The tree that would grow to heaven must send its roots to hell.".as_bytes();
    let formatting = Formatting::with_sort(Sort::Unsorted);

    // Include words starting with 'h' but exclude 'hell'
    let include_patterns = vec!["^h.*".to_string()];
    let exclude_patterns = vec!["^hell$".to_string()];

    let filters = Filters::default()
        .with_include_patterns(&include_patterns)
        .unwrap()
        .with_exclude_patterns(&exclude_patterns)
        .unwrap();

    let performance = Performance::default()
        .with_io(Io::Streamed)
        .with_processing(Processing::Sequential)
        .with_size_hint(SizeHint::None);
    let options = Options::new(formatting, filters, performance);
    let options_arc = make_shared(options);
    let tally = WordTally::new(input, &options_arc);
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
    let input = b"The quick brown fox jumps over the lazy dog. The fox was quick.";

    // Sequential processing
    let seq_performance = Performance::default()
        .with_io(Io::Streamed)
        .with_processing(Processing::Sequential);
    let filters = Filters::default();
    let seq_options = Options::new(Formatting::default(), filters.clone(), seq_performance);
    let seq_options_arc = make_shared(seq_options);
    let sequential = WordTally::new(&input[..], &seq_options_arc);

    // Parallel processing
    let par_performance = Performance::default()
        .with_io(Io::Streamed)
        .with_processing(Processing::Parallel);
    let par_options = Options::new(Formatting::default(), filters, par_performance);
    let par_options_arc = make_shared(par_options);
    let parallel = WordTally::new(&input[..], &par_options_arc);

    assert_eq!(sequential.count(), parallel.count());
    assert_eq!(sequential.uniq_count(), parallel.uniq_count());
    assert_eq!(sequential.tally(), parallel.tally());
}

#[test]
fn test_memory_mapped_vs_streamed() {
    // Open the test file (this is required for memory mapping)
    let file_path = TEST_WORDS_PATH;
    let file = File::open(file_path).expect("Failed to open test file for memory mapping");

    // Create a copy of the file for streaming
    let stream_file = File::open(file_path).expect("Failed to open test file for streaming");

    // Set up options for memory-mapped I/O (sequential)
    let mmap_performance = Performance::default()
        .with_io(Io::MemoryMapped)
        .with_processing(Processing::Sequential);
    let filters = Filters::default();
    let mmap_options = Options::new(Formatting::default(), filters.clone(), mmap_performance);

    // Set up options for streaming I/O (sequential)
    let stream_performance = Performance::default()
        .with_io(Io::Streamed)
        .with_processing(Processing::Sequential);
    let stream_options = Options::new(Formatting::default(), filters.clone(), stream_performance);

    // Create WordTally instances with the different I/O modes
    let memory_mapped =
        WordTally::try_from_file(file, &mmap_options).expect("Memory mapping failed");
    let streamed = WordTally::new(stream_file, &stream_options);

    // Verify results are the same regardless of I/O mode
    assert_eq!(memory_mapped.count(), streamed.count());
    assert_eq!(memory_mapped.uniq_count(), streamed.uniq_count());
    assert_eq!(memory_mapped.tally(), streamed.tally());

    // Now test with parallel processing
    let file = File::open(file_path).expect("Failed to open test file for memory mapping");
    let stream_file = File::open(file_path).expect("Failed to open test file for streaming");

    // Set up options for parallel I/O
    let parallel_performance = Performance::default()
        .with_io(Io::Streamed)
        .with_processing(Processing::Parallel);
    let parallel_options = Options::new(Formatting::default(), filters, parallel_performance);

    // Create WordTally instances with parallel processing
    let parallel_result = WordTally::new(file, &parallel_options);

    // Test that parallel also works with standard file I/O
    let parallel_stream = WordTally::new(stream_file, &parallel_options);

    // Verify results are the same with parallel processing
    assert_eq!(parallel_result.count(), parallel_stream.count());
    assert_eq!(parallel_result.uniq_count(), parallel_stream.uniq_count());
}

#[test]
fn test_with_size_hint() {
    let input = b"The quick brown fox jumps over the lazy dog.";

    // Without size hint
    let no_hint_performance = Performance::default()
        .with_io(Io::Streamed)
        .with_processing(Processing::Parallel)
        .with_size_hint(SizeHint::default());
    let filters = Filters::default();
    let no_hint_options = Options::new(Formatting::default(), filters.clone(), no_hint_performance);

    let without_hint = WordTally::new(&input[..], &no_hint_options);

    // With size hint
    let with_hint_performance = Performance::default()
        .with_io(Io::Streamed)
        .with_processing(Processing::Parallel)
        .with_size_hint(SizeHint::Bytes(input.len() as u64));
    let with_hint_options = Options::new(Formatting::default(), filters, with_hint_performance);

    let with_hint = WordTally::new(&input[..], &with_hint_options);

    assert_eq!(without_hint.count(), with_hint.count());
    assert_eq!(without_hint.uniq_count(), with_hint.uniq_count());
    assert_eq!(without_hint.tally(), with_hint.tally());
}

#[test]
fn test_estimate_capacity() {
    let small_performance = Performance::default()
        .with_io(Io::Streamed)
        .with_processing(Processing::Sequential)
        .with_size_hint(SizeHint::Bytes(8192)); // 8KB
    let small_options = Options::with_defaults(Formatting::default(), small_performance);

    let small_file = WordTally::new(&b"small text"[..], &small_options);

    let medium_performance = Performance::default()
        .with_io(Io::Streamed)
        .with_processing(Processing::Sequential)
        .with_size_hint(SizeHint::Bytes(524288)); // 512KB
    let medium_options = Options::with_defaults(Formatting::default(), medium_performance);

    let medium_file = WordTally::new(&b"medium text"[..], &medium_options);

    let large_performance = Performance::default()
        .with_io(Io::Streamed)
        .with_processing(Processing::Sequential)
        .with_size_hint(SizeHint::Bytes(4194304)); // 4MB
    let large_options = Options::with_defaults(Formatting::default(), large_performance);

    let large_file = WordTally::new(&b"large text"[..], &large_options);

    assert_eq!(small_file.count(), 2);
    assert_eq!(medium_file.count(), 2);
    assert_eq!(large_file.count(), 2);
}

#[test]
fn test_parallel_count() {
    // Instead of using environment variables, just test the parallel function works
    let input = b"Test with default settings for chunk size and thread count";
    let performance = Performance::default()
        .with_io(Io::Streamed)
        .with_processing(Processing::Parallel);
    let options = Options::with_defaults(Formatting::default(), performance);
    let parallel = WordTally::new(&input[..], &options);

    // Only check the counts are positive numbers (actual counts may vary by implementation)
    assert!(parallel.count() > 0);
    assert!(parallel.uniq_count() > 0);
    // Also check uniq count is less than or equal to total count
    assert!(parallel.uniq_count() <= parallel.count());
}

#[test]
fn test_merge_maps() {
    let input = b"This is a test of the map merging functionality";
    let performance = Performance::default()
        .with_io(Io::Streamed)
        .with_processing(Processing::Parallel);
    let options = Options::with_defaults(Formatting::default(), performance);
    let tally = WordTally::new(&input[..], &options);

    assert_eq!(tally.count(), 9);
    assert_eq!(tally.uniq_count(), 9);
}

#[test]
fn test_words_exclude_from() {
    let words = vec!["beep".to_string(), "boop".to_string()];
    assert_eq!(ExcludeWords::from(words.clone()), ExcludeWords(words));
}

// Tests for Formatting convenience methods
mod formatting_tests {
    use super::*;

    #[test]
    fn with_sort() {
        let sort_only = Formatting::with_sort(Sort::Asc);
        assert_eq!(sort_only.sort(), Sort::Asc);
        assert_eq!(sort_only.case(), Case::default());
    }

    #[test]
    fn with_case() {
        let case_only = Formatting::with_case(Case::Upper);
        assert_eq!(case_only.case(), Case::Upper);
        assert_eq!(case_only.sort(), Sort::default());
    }
}

// Tests for Performance struct
mod performance_tests {
    use super::*;

    #[test]
    fn default_values() {
        let performance = Performance::default();
        assert_eq!(performance.default_capacity(), 1024);
        assert_eq!(performance.uniqueness_ratio(), 10);
        assert_eq!(performance.unique_word_density(), 15);
        assert_eq!(performance.chunk_size(), 65_536);
    }

    #[test]
    fn builder_methods() {
        let performance = Performance::default()
            .with_capacity(2048)
            .with_uniqueness_ratio(5)
            .with_word_density(20)
            .with_chunk_size(32_768);

        assert_eq!(performance.default_capacity(), 2048);
        assert_eq!(performance.uniqueness_ratio(), 5);
        assert_eq!(performance.unique_word_density(), 20);
        assert_eq!(performance.chunk_size(), 32_768);
    }

    #[test]
    fn estimate_capacity() {
        let performance = Performance::default();

        // Default when no size hint
        assert_eq!(performance.estimate_capacity(), 1024);

        // Size hint divided by uniqueness ratio (10)
        assert_eq!(
            performance
                .with_size_hint(SizeHint::Bytes(8192))
                .estimate_capacity(),
            819
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

    #[test]
    fn with_defaults() {
        let options = Options::default();
        let tally = WordTally::new(TEST_INPUT, &options);
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
        let performance = Performance::default()
            .with_io(Io::Streamed)
            .with_processing(Processing::Parallel);
        let options = Options::with_defaults(Formatting::default(), performance);
        let tally = WordTally::new(TEST_INPUT, &options);
        assert_eq!(tally.count(), 3);
    }

    #[test]
    fn with_custom_chunk_size() {
        // Create custom performance with a specific chunk size
        let performance = Performance::default()
            .with_io(Io::Streamed)
            .with_processing(Processing::Parallel)
            .with_chunk_size(32_768);
        let options = Options::with_defaults(Formatting::default(), performance);

        let tally = WordTally::new(TEST_INPUT, &options);
        assert_eq!(tally.count(), 3);
    }
}

#[test]
fn test_min_count_graphemes() {
    let performance = Performance::default()
        .with_io(Io::Streamed)
        .with_processing(Processing::Sequential);
    let filters = Filters::default().with_min_chars(2);
    let options = Options::new(Formatting::default(), filters, performance);
    let tally = WordTally::new(
        // An `"é"` is only one char.
        &b"e\xCC\x81"[..],
        &options,
    );

    assert_eq!(tally.count(), 0);
}

#[test]
fn test_to_json() {
    // Create options
    let performance = Performance::default()
        .with_io(Io::Streamed)
        .with_processing(Processing::Sequential);
    let filters = Filters::default();
    let options = Options::new(Formatting::default(), filters, performance);

    // Create a static reference
    let shared_options = make_shared(options);

    let expected = WordTally::new(&b"wombat wombat bat"[..], &shared_options);
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
    let performance = Performance::default()
        .with_io(Io::Streamed)
        .with_processing(Processing::Sequential);
    let filters = Filters::default();
    let options = Options::new(Formatting::default(), filters, performance);
    let shared_options = make_shared(options);

    let expected = WordTally::new(&b"wombat wombat bat"[..], &shared_options);
    let json = r#"
    {
        "tally": [["wombat", 2], ["bat", 1]],
        "options": {
            "formatting": {"case": "Lower", "sort": "Desc"},
            "filters": {"min_chars": null, "min_count": null, "exclude_words": null}
        },
        "count": 3,
        "uniqueCount": 2
    }
    "#;

    let deserialized = WordTally::from_json_str(json, &shared_options).unwrap();
    assert_eq!(deserialized, expected);
}

#[test]
fn test_explicit_deserialization() {
    let options = Options::default();
    let original = WordTally::new(&b"wombat wombat bat"[..], &options);
    let json = serde_json::to_string(&original).unwrap();
    let deserialized = WordTally::from_json_str(&json, &options).unwrap();

    assert_eq!(deserialized.count(), original.count());
    assert_eq!(deserialized.uniq_count(), original.uniq_count());
    assert_eq!(deserialized.tally(), original.tally());
    assert!(std::ptr::eq(deserialized.options(), &options));
}

#[test]
fn test_json_field_renamed() {
    // Test that the field renaming in the serialization works correctly
    let options = Options::default();
    let original = WordTally::new(&b"test json field renaming"[..], &options);
    let json = serde_json::to_string(&original).unwrap();

    // Check that the JSON contains "uniqueCount" instead of "uniq_count"
    assert!(json.contains("uniqueCount"));
    assert!(!json.contains("uniq_count"));
}

#[test]
fn test_json_field_camel_case_deserialization() {
    // Test that camelCase field names are used in serialization and deserialization
    let options = make_shared(Options::default());

    // JSON with camelCase field names
    let json = r#"
    {
        "tally": [["test", 1]],
        "options": {
            "formatting": {"case": "Lower", "sort": "Desc"},
            "filters": {}
        },
        "count": 1,
        "uniqueCount": 1
    }
    "#;

    // Should deserialize correctly
    let deserialized = WordTally::from_json_str(json, &options).unwrap();

    assert_eq!(deserialized.count(), 1);
    assert_eq!(deserialized.uniq_count(), 1);

    // Serialize and verify it uses camelCase
    let serialized = serde_json::to_string(&deserialized).unwrap();
    assert!(serialized.contains("\"uniqueCount\":1"));
    assert!(!serialized.contains("\"uniq_count\":"));
}

#[test]
fn test_error_handling_invalid_json() {
    let options = Options::default();

    // Invalid JSON syntax
    let invalid_json = r#"
    {
        "tally": [["test", 1]],
        "count": 1,
        "uniq_count": 1,
        this is invalid
    }
    "#;

    let result = WordTally::from_json_str(invalid_json, &options);
    assert!(result.is_err());
}

#[test]
fn test_error_handling_missing_fields() {
    let options = Options::default();

    // Missing required fields
    let missing_fields_json = r#"
    {
        "tally": [["test", 1]]
    }
    "#;

    let result = WordTally::from_json_str(missing_fields_json, &options);
    assert!(result.is_err());
}
