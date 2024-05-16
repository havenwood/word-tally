use clap_stdin::FileOrStdin;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::str::FromStr;
use word_tally::*;

const WORDS_PATH: &str = "tests/files/words.txt";

struct Fields<'a> {
    count: u64,
    uniq_count: usize,
    avg: f64,
    tally: Vec<(&'a str, u64)>,
}

fn word_tally_test(case: Case, sort: Sort, fields: Fields) {
    if let Ok(file_or_stdin) = FileOrStdin::from_str(WORDS_PATH) {
        let word_tally = WordTally::new(&file_or_stdin, case, sort).unwrap();

        assert_eq!(word_tally.count(), fields.count);
        assert_eq!(word_tally.uniq_count(), fields.uniq_count);
        assert_eq!(word_tally.avg().unwrap(), fields.avg);

        let expected_tally: Vec<(String, u64)> = fields
            .tally
            .iter()
            .map(|(word, count)| ((*word).to_string(), *count))
            .collect();
        assert_eq!(word_tally.tally(), &expected_tally);
    }
}

#[test]
fn case_insensitive_desc_order() {
    word_tally_test(
        Case::Lower,
        Sort::Desc,
        Fields {
            count: 45,
            uniq_count: 5,
            avg: 9.0,
            tally: vec![("c", 15), ("d", 11), ("123", 9), ("b", 7), ("a", 3)],
        },
    );

    word_tally_test(
        Case::Upper,
        Sort::Desc,
        Fields {
            count: 45,
            uniq_count: 5,
            avg: 9.0,
            tally: vec![("C", 15), ("D", 11), ("123", 9), ("B", 7), ("A", 3)],
        },
    );
}

#[test]
fn case_insensitive_asc_order() {
    word_tally_test(
        Case::Lower,
        Sort::Asc,
        Fields {
            count: 45,
            uniq_count: 5,
            avg: 9.0,
            tally: vec![("a", 3), ("b", 7), ("123", 9), ("d", 11), ("c", 15)],
        },
    );
}

#[test]
fn case_sensitive_desc_order() {
    word_tally_test(
        Case::Original,
        Sort::Desc,
        Fields {
            count: 45,
            uniq_count: 9,
            avg: 5.0,
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
fn case_sensitive_asc_order() {
    word_tally_test(
        Case::Original,
        Sort::Asc,
        Fields {
            count: 45,
            uniq_count: 9,
            avg: 5.0,
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
    )
}

#[test]
fn equality_and_hashing() {
    if let Ok(file_or_stdin) = FileOrStdin::from_str("tests/files/words.txt") {
        let a1 = WordTally::new(&file_or_stdin, Case::Original, Sort::Desc).unwrap();
        let a2 = WordTally::new(&file_or_stdin, Case::Original, Sort::Desc).unwrap();
        let b1 = WordTally::new(&file_or_stdin, Case::Lower, Sort::Desc).unwrap();
        let c1 = WordTally::new(&file_or_stdin, Case::Lower, Sort::Unsorted).unwrap();
        let d1 = WordTally::new(&file_or_stdin, Case::Lower, Sort::Asc).unwrap();

        assert_eq!(a1, a2);
        assert_eq!(a2, a1);
        assert_ne!(a1, b1);
        assert_ne!(b1, a1);
        assert_ne!(a1, c1);
        assert_ne!(c1, a1);
        assert_ne!(a1, d1);
        assert_ne!(d1, a1);

        let mut a1_a2 = DefaultHasher::new();
        a1.hash(&mut a1_a2);
        a2.hash(&mut a1_a2);

        let mut a2_a1 = DefaultHasher::new();
        a2.hash(&mut a2_a1);
        a1.hash(&mut a2_a1);

        let mut a1_b1 = DefaultHasher::new();
        a1.hash(&mut a1_b1);
        b1.hash(&mut a1_b1);

        let mut b1_a1 = DefaultHasher::new();
        b1.hash(&mut b1_a1);
        a1.hash(&mut b1_a1);

        assert_eq!(a1_a2.finish(), a2_a1.finish());
        assert_ne!(a1_b1.finish(), b1_a1.finish());
    }
}

#[test]
fn vec_from() {
    if let Ok(file_or_stdin) = FileOrStdin::from_str(WORDS_PATH) {
        let tally = WordTally::new(&file_or_stdin, Case::Lower, Sort::Desc).unwrap();

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
}
