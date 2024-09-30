use std::fs::File;
use std::hash::{DefaultHasher, Hash, Hasher};
use word_tally::{
    Case, Filters, MinChars, MinCount, Options, Sort, WordTally, WordsExclude, WordsOnly,
};

const TEST_WORDS_PATH: &str = "tests/files/words.txt";

struct ExpectedFields<'a> {
    count: u64,
    uniq_count: usize,
    avg: Option<f64>,
    tally: Vec<(&'a str, u64)>,
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
    assert_eq!(word_tally.avg(), fields.avg);

    let expected_tally = fields
        .tally
        .iter()
        .map(|(word, count)| (Box::from(*word), *count))
        .collect::<Vec<_>>()
        .into_boxed_slice();
    assert_eq!(word_tally.tally(), expected_tally);
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
            avg: Some(9.0),
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
            avg: Some(9.0),
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
            avg: None,
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
            avg: Some(9.0),
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
            avg: Some(15.0),
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
            avg: Some(9.0),
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
            avg: Some(9.0),
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
            avg: Some(9.0),
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
            avg: Some(9.0),
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
            avg: Some(5.0),
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
            avg: Some(5.0),
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
fn test_excluding_words() {
    let input = "The tree that would grow to heaven must send its roots to hell.".as_bytes();
    let excluded_words = vec!["Heaven".to_string(), "Hell".to_string()];
    let options = Options {
        sort: Sort::Unsorted,
        ..Options::default()
    };
    let filters = Filters {
        words_exclude: Some(WordsExclude(excluded_words)),
        ..Filters::default()
    };
    let tally = WordTally::new(input, options, filters);
    let result = tally.tally();

    assert!(result.iter().any(|(word, _)| word.as_ref() == "tree"));
    assert!(!result.iter().any(|(word, _)| word.as_ref() == "heaven"));
    assert!(!result.iter().any(|(word, _)| word.as_ref() == "hell"));
}

#[test]
fn test_only_words() {
    let input = "One must still have chaos in oneself to be able to give birth to a dancing star. I tell you: you have chaos in yourselves.".as_bytes();
    let only = vec!["chaos".to_string(), "star".to_string()];
    let options = Options {
        case: Case::Lower,
        sort: Sort::Desc,
    };
    let filters = Filters {
        words_only: Some(WordsOnly(only)),
        ..Filters::default()
    };

    let tally = WordTally::new(input, options, filters);
    let result = tally.tally();

    let expected = vec![(Box::from("chaos"), 2), (Box::from("star"), 1)].into_boxed_slice();

    assert_eq!(result, expected);
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
    let raw = 43_u64;
    assert_eq!(MinCount::from(raw), MinCount(43));
}

#[test]
fn test_words_exclude_from() {
    let excluded = vec!["beep".to_string(), "boop".to_string()];
    assert_eq!(WordsExclude::from(excluded.clone()), WordsExclude(excluded));
}

#[test]
fn test_words_only_from() {
    let only = vec!["bep".to_string(), "bop".to_string()];
    assert_eq!(WordsOnly::from(only.clone()), WordsOnly(only));
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

#[cfg(feature = "serde")]
#[test]
fn test_to_json() {
    let expected = WordTally::new(
        &b"wombat wombat bat"[..],
        Options::default(),
        Filters::default(),
    );
    let serialized = serde_json::to_string(&expected).unwrap();

    let expected_json = r#"{"tally":[["wombat",2],["bat",1]],"count":3,"uniq_count":2,"avg":1.5}"#;
    assert_eq!(serialized, expected_json);
}

#[cfg(feature = "serde")]
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
        "count": 3,
        "uniq_count": 2,
        "avg": 1.5
    }
    "#;

    let deserialized: WordTally = serde_json::from_str(json).unwrap();
    assert_eq!(deserialized, expected);
}
