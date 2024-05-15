use clap_stdin::FileOrStdin;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::str::FromStr;
use word_tally::WordTally;

#[test]
fn sorted_case_insensitive() {
    if let Ok(file_or_stdin) = FileOrStdin::from_str("tests/files/words.txt") {
        let word_tally = WordTally::new(&file_or_stdin, false, true).unwrap();

        assert_eq!(word_tally.count(), 45);
        assert_eq!(word_tally.uniq_count(), 5);
        assert_eq!(word_tally.avg().unwrap(), 9.0);
        assert_eq!(
            word_tally.tally(),
            &vec![
                ("c".to_string(), 15),
                ("d".to_string(), 11),
                ("123".to_string(), 9),
                ("b".to_string(), 7),
                ("a".to_string(), 3)
            ]
        );
    }
}

#[test]
fn sorted_case_sensitive() {
    if let Ok(file_or_stdin) = FileOrStdin::from_str("tests/files/words.txt") {
        let word_tally = WordTally::new(&file_or_stdin, true, true).unwrap();

        assert_eq!(word_tally.count(), 45);
        assert_eq!(word_tally.uniq_count(), 9);
        assert_eq!(word_tally.avg().unwrap(), 5.0);
        assert_eq!(
            word_tally.tally(),
            &vec![
                ("123".to_string(), 9),
                ("C".to_string(), 8),
                ("c".to_string(), 7),
                ("D".to_string(), 6),
                ("d".to_string(), 5),
                ("B".to_string(), 4),
                ("b".to_string(), 3),
                ("A".to_string(), 2),
                ("a".to_string(), 1)
            ]
        );
    }
}

#[test]
fn equality_and_hashing() {
    if let Ok(file_or_stdin) = FileOrStdin::from_str("tests/files/words.txt") {
        let a1 = WordTally::new(&file_or_stdin, true, true).unwrap();
        let a2 = WordTally::new(&file_or_stdin, true, true).unwrap();
        let b1 = WordTally::new(&file_or_stdin, false, true).unwrap();
        let c1 = WordTally::new(&file_or_stdin, true, false).unwrap();

        assert_eq!(a1, a2);
        assert_eq!(a2, a1);
        assert_ne!(a1, b1);
        assert_ne!(b1, a1);
        assert_ne!(a1, c1);
        assert_ne!(c1, a1);

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
    if let Ok(file_or_stdin) = FileOrStdin::from_str("tests/files/words.txt") {
        let tally = WordTally::new(&file_or_stdin, false, true).unwrap();

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
