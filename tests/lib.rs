use clap_stdin::FileOrStdin;
use std::str::FromStr;
use word_tally::WordTally;

#[test]
fn sorted_case_insensitive() {
    if let Ok(file_or_stdin) = FileOrStdin::from_str("tests/files/words.txt") {
        let word_tally = WordTally::new(&file_or_stdin, false, true);

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
        let word_tally = WordTally::new(&file_or_stdin, true, true);

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
