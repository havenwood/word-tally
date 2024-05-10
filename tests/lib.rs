use clap_stdin::FileOrStdin;
use std::str::FromStr;
use word_tally::WordTally;

#[test]
fn sorted_case_insensitive() {
    if let Ok(file_or_stdin) = FileOrStdin::from_str("tests/files/words.txt") {
        let word_tally = WordTally::new(&file_or_stdin, false, true);

        assert_eq!(word_tally.count(), 21);
        assert_eq!(word_tally.uniq_count().unwrap(), 6);
        assert_eq!(word_tally.avg().unwrap(), 3.5);
        assert_eq!(
            word_tally.tally(),
            &vec![
                ("f".to_string(), 6),
                ("e".to_string(), 5),
                ("d".to_string(), 4),
                ("c".to_string(), 3),
                ("b".to_string(), 2),
                ("a".to_string(), 1),
            ]
        );
    }
}
