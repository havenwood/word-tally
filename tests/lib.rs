use std::fs::File;
use std::hash::{DefaultHasher, Hash, Hasher};
use word_tally::{Case, Chars, Count, Filters, Sort, WordTally, Words};

const TEST_WORDS_PATH: &str = "tests/files/words.txt";

struct ExpectedFields<'a> {
    count: u64,
    uniq_count: usize,
    avg: Option<f64>,
    tally: Vec<(&'a str, u64)>,
}

fn word_tally(case: Case, sort: Sort, filters: Filters) -> WordTally {
    let input = File::open(TEST_WORDS_PATH)
        .expect("Expected test words file (`files/words.txt`) to be readable.");

    WordTally::new(input, case, sort, filters)
}

fn word_tally_test(case: Case, sort: Sort, filters: Filters, fields: &ExpectedFields<'_>) {
    let word_tally = word_tally(case, sort, filters);
    assert_eq!(word_tally.count(), fields.count);
    assert_eq!(word_tally.uniq_count(), fields.uniq_count);
    assert_eq!(word_tally.avg(), fields.avg);

    let expected_tally: Vec<(String, u64)> = fields
        .tally
        .iter()
        .map(|(word, count)| ((*word).to_string(), *count))
        .collect();
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
            chars: Chars::min(3),
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
            chars: Chars::min(4),
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
            count: Count::min(15),
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
        .map(|&(case, sort)| word_tally(case, sort, Filters::default()))
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
    let tally = word_tally(Case::Lower, Sort::Desc, Filters::default());

    assert_eq!(
        Vec::from(tally),
        vec![
            ("c".to_string(), 15),
            ("d".to_string(), 11),
            ("123".to_string(), 9),
            ("b".to_string(), 7),
            ("a".to_string(), 3)
        ]
    );
}

#[test]
fn test_excluding_words() {
    let input = "The tree that would grow to heaven must send its roots to hell.".as_bytes();
    let excluded_words = vec!["Heaven".to_string(), "Hell".to_string()];
    let filters = Filters {
        words: Words::exclude(Some(excluded_words)),
        ..Filters::default()
    };
    let tally = WordTally::new(input, Case::Lower, Sort::Unsorted, filters);
    let result = tally.tally();

    assert!(result.iter().any(|(word, _)| word == "tree"));
    assert!(!result.iter().any(|(word, _)| word == "heaven"));
    assert!(!result.iter().any(|(word, _)| word == "hell"));
}
