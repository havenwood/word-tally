//! Tests for encoding functionality (ASCII and Unicode modes)

use assert_cmd::Command;
use hashbrown::{HashMap, HashSet};
use word_tally::{Case, TallyMap, options::encoding::Encoding};

fn word_tally() -> Command {
    Command::cargo_bin("word-tally").expect("binary should be available")
}

fn parse_output(output: &[u8]) -> HashSet<(String, usize)> {
    simdutf8::compat::from_utf8(output)
        .expect("output should be valid UTF-8")
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            assert_eq!(parts.len(), 2, "Invalid output line: {line}");
            (
                parts[0].to_string(),
                parts[1].parse().expect("should be valid count"),
            )
        })
        .collect()
}

fn test_encoding_parity(input: &str, args: &[&str]) {
    let unicode_output = word_tally()
        .arg("--encoding=unicode")
        .args(args)
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let ascii_output = word_tally()
        .arg("--encoding=ascii")
        .args(args)
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let unicode_words = parse_output(&unicode_output);
    let ascii_words = parse_output(&ascii_output);

    assert_eq!(
        unicode_words, ascii_words,
        "Encoding outputs should match for input: {input}"
    );
}

// Segmenter tests

#[test]
fn test_basic_segmentation() {
    fn test_segmenter(encoding: Encoding, expected_words: &[&str]) {
        let mut tally = TallyMap::new();
        tally
            .add_words("Hello, world!", Case::Original, encoding)
            .expect("segmentation should succeed");

        let words: Vec<String> = tally.into_iter().map(|(w, _)| w.into()).collect();
        assert_eq!(words.len(), expected_words.len());
        for word in expected_words {
            assert!(words.contains(&(*word).to_string()));
        }
    }

    test_segmenter(Encoding::Unicode, &["Hello", "world"]);
    test_segmenter(Encoding::Ascii, &["Hello", "world"]);
}

#[test]
fn test_apostrophes() {
    let mut tally = TallyMap::new();
    tally
        .add_words("don't can't", Case::Original, Encoding::Ascii)
        .expect("should handle apostrophes");

    let words: Vec<String> = tally.into_iter().map(|(w, _)| w.into()).collect();
    assert_eq!(words.len(), 2);
    assert!(words.contains(&"don't".to_string()));
    assert!(words.contains(&"can't".to_string()));
}

#[test]
fn test_ascii_rejects_non_ascii() {
    let mut tally = TallyMap::new();
    let result = tally.add_words("café", Case::Original, Encoding::Ascii);
    assert!(result.is_err());
    assert!(
        result
            .expect_err("should have non-ASCII error")
            .to_string()
            .contains("non-ASCII")
    );
}

#[test]
fn test_case_normalization() {
    fn test_case(encoding: Encoding, case: Case, expected: &[&str]) {
        let mut tally = TallyMap::new();
        tally
            .add_words("Hello WORLD", case, encoding)
            .expect("case normalization should succeed");

        let words: Vec<String> = tally.into_iter().map(|(w, _)| w.into()).collect();
        assert_eq!(words.len(), expected.len());
        for word in expected {
            assert!(words.contains(&(*word).to_string()));
        }
    }

    test_case(Encoding::Unicode, Case::Lower, &["hello", "world"]);
    test_case(Encoding::Ascii, Case::Upper, &["HELLO", "WORLD"]);
}

// Unicode handling tests

#[test]
fn test_unicode_words() {
    let output = word_tally()
        .arg("--encoding=unicode")
        .arg("--case=lower")
        .arg("--sort=desc")
        .write_stdin("café über señor hello world café")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let word_counts: HashMap<&str, usize> = simdutf8::compat::from_utf8(&output)
        .expect("output should be valid UTF-8")
        .lines()
        .filter_map(|line| {
            line.split_once(char::is_whitespace).map(|(word, count)| {
                (
                    word.trim(),
                    count.trim().parse().expect("should be valid count"),
                )
            })
        })
        .collect();

    assert_eq!(word_counts["café"], 2);
    assert_eq!(word_counts["über"], 1);
    assert_eq!(word_counts["señor"], 1);
    assert_eq!(word_counts["hello"], 1);
    assert_eq!(word_counts["world"], 1);
}

#[test]
fn test_unicode_min_chars() {
    let output = word_tally()
        .arg("--min-chars=4")
        .write_stdin("a café naive naïve")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let words: Vec<&str> = simdutf8::compat::from_utf8(&output)
        .expect("output should be valid UTF-8")
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| line.split_whitespace().next().expect("should have word"))
        .collect();

    assert!(!words.contains(&"a"));
    assert!(words.contains(&"café"));
    assert!(words.contains(&"naive"));
    assert!(words.contains(&"naïve"));
}

// ASCII/Unicode parity tests (for ASCII-only input)

#[test]
fn test_ascii_parity_basic() {
    let test_cases = [
        ("hello world test data hello", &[][..]),
        ("don't can't it's we're I'll they'll", &[]),
        ("test123 456test hello2world 2fast2furious", &[]),
        ("hello, world! How are you? I'm fine... Really!", &[]),
    ];

    for (input, args) in &test_cases {
        test_encoding_parity(input, args);
    }
}

#[test]
fn test_case_handling_parity() {
    let input = "Hello WORLD hello World HELLO world";
    test_encoding_parity(input, &["--case=original"]);
    test_encoding_parity(input, &["--case=lower"]);
}

#[test]
fn test_edge_cases_parity() {
    let test_cases = [
        ("word", &[][..]),
        ("", &[]),
        ("   ", &[]),
        ("a b c", &[]),
        ("123 456", &[]),
        ("test's", &[]),
        ("well-known", &[]),
        ("C:\\path\\file", &[]),
        ("test...test", &[]),
        ("test--test", &[]),
        ("'twas rock'n'roll y'all ma'am", &[]),
    ];

    for (input, args) in &test_cases {
        test_encoding_parity(input, args);
    }
}

// Documented differences between ASCII and Unicode

#[test]
fn test_unicode_accepts_non_ascii() {
    let output = word_tally()
        .arg("--encoding=unicode")
        .write_stdin("café naïve")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let unicode_str = String::from_utf8_lossy(&output);
    assert!(unicode_str.contains("café"));
    assert!(unicode_str.contains("naïve"));
}

#[test]
fn test_ascii_rejects_non_ascii_cli() {
    word_tally()
        .arg("--encoding=ascii")
        .write_stdin("café naïve")
        .assert()
        .failure()
        .code(65);
}

#[test]
fn test_ascii_word_boundaries() {
    let output = word_tally()
        .arg("--encoding=ascii")
        .arg("--sort=asc")
        .write_stdin("test@example.com, hello-world, 3.14, 'quoted', test's, rock'n'roll")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let words: Vec<&str> = simdutf8::compat::from_utf8(&output)
        .expect("output should be valid UTF-8")
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| line.split_whitespace().next().expect("should have word"))
        .collect();

    // ASCII mode splits on non-alphanumeric except apostrophes
    let expected_words = [
        "test",
        "example",
        "com",
        "hello",
        "world",
        "3",
        "14",
        "quoted'",
        "test's",
        "rock'n'roll",
    ];

    for expected in &expected_words {
        assert!(words.contains(expected), "Missing word: {expected}");
    }
}
