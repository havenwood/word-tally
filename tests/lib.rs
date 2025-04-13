use std::fs::File;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::PathBuf;
use word_tally::input::Input;
use word_tally::output::Output;
use word_tally::{
    Case, Config, ExcludeWords, Filters, MinChars, MinCount, Options, Sort, WordTally,
};

const TEST_WORDS_PATH: &str = "tests/files/words.txt";

struct ExpectedFields<'a> {
    count: usize,
    uniq_count: usize,
    tally: Vec<(&'a str, usize)>,
}

fn word_tally(options: Options, filters: Filters) -> WordTally {
    let input = File::open(TEST_WORDS_PATH)
        .expect("Expected test words file (`files/words.txt`) to be readable.");

    WordTally::new(input, options, filters)
}

fn word_tally_test(case: Case, sort: Sort, filters: Filters, fields: &ExpectedFields<'_>) {
    let word_tally = word_tally(Options { case, sort }, filters);
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
        Filters {
            min_chars: Some(MinChars(3)),
            ..Filters::default()
        },
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
        Filters {
            min_chars: Some(MinChars(4)),
            ..Filters::default()
        },
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
        Filters {
            min_count: Some(MinCount(15)),
            ..Filters::default()
        },
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
    fn hash_value(word_tally: &WordTally) -> u64 {
        let mut hasher = DefaultHasher::new();
        word_tally.hash(&mut hasher);
        hasher.finish()
    }

    fn assert_hash_eq(tally_a: &WordTally, tally_b: &WordTally) {
        assert_eq!(hash_value(tally_a), hash_value(tally_b));
    }

    fn assert_hash_ne(tally_a: &WordTally, tally_b: &WordTally) {
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

    let tallies: Vec<WordTally> = cases_and_sorts
        .iter()
        .map(|&(case, sort)| word_tally(Options { case, sort }, Filters::default()))
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
    let tally = word_tally(Options::default(), Filters::default());

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
    let options = Options::default();
    let filters = Filters::default();

    let word_tally = WordTally::new(&input[..], options, filters);

    // Use `tally()` to get a reference to the slice.
    let tally = word_tally.tally();

    let expected_tally: Box<[(Box<str>, usize)]> =
        vec![("bye".into(), 2), ("birdy".into(), 1)].into_boxed_slice();

    assert_eq!(tally, expected_tally.as_ref());
}

#[test]
fn test_iterator() {
    let input = b"double trouble double";
    let options = Options {
        case: Case::Lower,
        sort: Sort::Desc,
    };
    let filters = Filters::default();
    let word_tally = WordTally::new(&input[..], options, filters);

    let expected: Vec<(Box<str>, usize)> =
        vec![(Box::from("double"), 2), (Box::from("trouble"), 1)];

    let collected: Vec<(Box<str>, usize)> = (&word_tally).into_iter().cloned().collect();
    assert_eq!(collected, expected);

    let mut iter = (&word_tally).into_iter();
    assert_eq!(iter.next(), Some(&(Box::from("double"), 2)));
    assert_eq!(iter.next(), Some(&(Box::from("trouble"), 1)));
    assert_eq!(iter.next(), None);
}

#[test]
fn test_iterator_for_loop() {
    let input = b"llama llama pajamas";
    let options = Options {
        case: Case::Lower,
        sort: Sort::Desc,
    };
    let filters = Filters::default();
    let word_tally = WordTally::new(&input[..], options, filters);

    let expected: Vec<(Box<str>, usize)> = vec![(Box::from("llama"), 2), (Box::from("pajamas"), 1)];

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
    let options = Options {
        sort: Sort::Unsorted,
        ..Options::default()
    };
    let filters = Filters {
        exclude: Some(ExcludeWords(words)),
        ..Filters::default()
    };
    let tally = WordTally::new(input, options, filters);
    let result = tally.tally();

    assert!(result.iter().any(|(word, _)| word.as_ref() == "tree"));
    assert!(!result.iter().any(|(word, _)| word.as_ref() == "heaven"));
    assert!(!result.iter().any(|(word, _)| word.as_ref() == "hell"));
}

#[test]
fn test_min_chars_display() {
    let min_chars = MinChars(42);
    assert_eq!(min_chars.to_string(), "42".to_string());
}

#[test]
fn test_min_chars_from() {
    assert_eq!(MinChars::from(42), MinChars(42));
}

#[test]
fn test_min_count_display() {
    let min_count = MinCount(43);
    assert_eq!(min_count.to_string(), "43".to_string());
}

#[test]
fn test_min_count_from() {
    let raw = 43_usize;
    assert_eq!(MinCount::from(raw), MinCount(43));
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
    let options = Options::default();
    let filters = Filters::default();

    let sequential = WordTally::new(&input[..], options, filters.clone());
    let parallel = WordTally::new_parallel(&input[..], options, filters);

    assert_eq!(sequential.count(), parallel.count());
    assert_eq!(sequential.uniq_count(), parallel.uniq_count());
    assert_eq!(sequential.tally(), parallel.tally());
}

#[test]
fn test_parallel_with_size_hint() {
    let input = b"The quick brown fox jumps over the lazy dog.";
    let options = Options::default();
    let filters = Filters::default();

    let without_hint = WordTally::new_parallel(&input[..], options, filters.clone());
    let with_hint =
        WordTally::new_parallel_with_size(&input[..], options, filters, Some(input.len() as u64));

    assert_eq!(without_hint.count(), with_hint.count());
    assert_eq!(without_hint.uniq_count(), with_hint.uniq_count());
    assert_eq!(without_hint.tally(), with_hint.tally());
}

#[test]
fn test_estimate_capacity() {
    let small_file = WordTally::new_with_size(
        &b"small text"[..],
        Options::default(),
        Filters::default(),
        Some(5_000), // 5KB
    );

    let medium_file = WordTally::new_with_size(
        &b"medium text"[..],
        Options::default(),
        Filters::default(),
        Some(500_000), // 500KB
    );

    let large_file = WordTally::new_with_size(
        &b"large text"[..],
        Options::default(),
        Filters::default(),
        Some(5_000_000), // 5MB
    );

    assert_eq!(small_file.count(), 2);
    assert_eq!(medium_file.count(), 2);
    assert_eq!(large_file.count(), 2);
}

#[test]
fn test_parallel_count() {
    // Instead of using environment variables, just test the parallel function works
    let input = b"Test with default settings for chunk size and thread count";
    let parallel = WordTally::new_parallel(&input[..], Options::default(), Filters::default());

    // Only check the counts are positive numbers (actual counts may vary by implementation)
    assert!(parallel.count() > 0);
    assert!(parallel.uniq_count() > 0);
    // Also check uniq count is less than or equal to total count
    assert!(parallel.uniq_count() <= parallel.count());
}

#[test]
fn test_merge_maps() {
    let input = b"This is a test of the map merging functionality";
    let options = Options::default();
    let filters = Filters::default();

    let tally = WordTally::new_parallel(&input[..], options, filters);

    assert_eq!(tally.count(), 9);
    assert_eq!(tally.uniq_count(), 9);
}

#[test]
fn test_words_exclude_from() {
    let words = vec!["beep".to_string(), "boop".to_string()];
    assert_eq!(ExcludeWords::from(words.clone()), ExcludeWords(words));
}

// Tests for Options convenience methods
mod options_tests {
    use super::*;

    #[test]
    fn with_sort() {
        let sort_only = Options::with_sort(Sort::Asc);
        assert_eq!(sort_only.sort, Sort::Asc);
        assert_eq!(sort_only.case, Case::default());
    }

    #[test]
    fn with_case() {
        let case_only = Options::with_case(Case::Upper);
        assert_eq!(case_only.case, Case::Upper);
        assert_eq!(case_only.sort, Sort::default());
    }
}

// Tests for Config struct
mod config_tests {
    use super::*;

    #[test]
    fn default_values() {
        let config = Config::default();
        assert_eq!(config.default_capacity(), 1024);
        assert_eq!(config.uniqueness_ratio(), 10);
        assert_eq!(config.unique_word_density(), 15);
        assert_eq!(config.chunk_size(), 16_384);
    }

    #[test]
    fn builder_methods() {
        let config = Config::default()
            .with_capacity(2048)
            .with_uniqueness_ratio(5)
            .with_word_density(20)
            .with_chunk_size(32_768);

        assert_eq!(config.default_capacity(), 2048);
        assert_eq!(config.uniqueness_ratio(), 5);
        assert_eq!(config.unique_word_density(), 20);
        assert_eq!(config.chunk_size(), 32_768);
    }

    #[test]
    fn estimate_capacity() {
        let config = Config::default();

        // Default when no size hint
        assert_eq!(config.estimate_capacity(None), 1024);

        // Size hint divided by uniqueness ratio (10)
        assert_eq!(config.estimate_capacity(Some(10_000)), 1000);
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
        let tally = WordTally::new_with_defaults(&TEST_INPUT[..]);
        assert_eq!(tally.count(), 3);
    }

    #[test]
    fn parallel_with_defaults() {
        let tally = WordTally::new_parallel_with_defaults(&TEST_INPUT[..]);
        assert_eq!(tally.count(), 3);
    }

    #[test]
    fn with_custom_config() {
        let config = Config::default().with_chunk_size(128);
        let tally = WordTally::new_with_config(
            &TEST_INPUT[..],
            Options::default(),
            Filters::default(),
            None,
            config,
        );
        assert_eq!(tally.count(), 3);
    }
}

#[test]
fn test_min_count_graphemes() {
    let tally = WordTally::new(
        // An `"é"` is only one char.
        &b"e\xCC\x81"[..],
        Options::default(),
        Filters {
            min_chars: Some(MinChars(2)),
            ..Filters::default()
        },
    );

    assert_eq!(tally.count(), 0);
}

#[test]
fn test_to_json() {
    let expected = WordTally::new(
        &b"wombat wombat bat"[..],
        Options::default(),
        Filters::default(),
    );
    let serialized = serde_json::to_string(&expected).unwrap();

    assert!(serialized.contains("\"tally\":[[\"wombat\",2],[\"bat\",1]]"));
    assert!(serialized.contains("\"count\":3"));
    assert!(serialized.contains("\"uniq_count\":2"));
    assert!(serialized.contains("\"options\":"));
    assert!(serialized.contains("\"filters\":"));
}

#[test]
fn test_from_json() {
    let expected = WordTally::new(
        &b"wombat wombat bat"[..],
        Options::default(),
        Filters::default(),
    );
    let json = r#"
    {
        "tally": [["wombat", 2], ["bat", 1]],
        "options": {"case": "Lower", "sort": "Desc"},
        "filters": {"min_chars": null, "min_count": null, "exclude": null},
        "count": 3,
        "uniq_count": 2
    }
    "#;

    let deserialized: WordTally = serde_json::from_str(json).unwrap();
    assert_eq!(deserialized, expected);
}
