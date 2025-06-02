//! Tests for encoding functionality (ASCII and Unicode modes)

use std::collections::{HashMap, HashSet};

use assert_cmd::Command;
use word_tally::{Case, TallyMap};

fn word_tally() -> Command {
    Command::cargo_bin("word-tally").expect("word-tally binary should be available")
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
                parts[1]
                    .parse::<usize>()
                    .expect("second part should be a valid count"),
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

fn test_encoding_parity_batch(test_cases: &[(&str, &[&str])]) {
    for (input, args) in test_cases {
        test_encoding_parity(input, args);
    }
}

// =============================================================================
// ASCII mode validation tests
// =============================================================================

#[test]
fn test_add_words_ascii() {
    let mut tally = TallyMap::new();

    // Valid ASCII text
    let result = tally.add_words_ascii("Hello world", Case::Lower);
    assert!(result.is_ok());
    assert_eq!(tally.len(), 2);

    // Non-ASCII text should fail
    let mut tally2 = TallyMap::new();
    let result = tally2.add_words_ascii("café", Case::Lower);
    assert!(result.is_err());

    // Test apostrophes are handled
    let mut tally3 = TallyMap::new();
    let result = tally3.add_words_ascii("don't can't", Case::Lower);
    assert!(result.is_ok());
    assert_eq!(tally3.len(), 2);
}

// =============================================================================
// Unicode handling tests
// =============================================================================

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
            line.split_once(char::is_whitespace)
                .map(|(word, count)| (word.trim(), count.trim().parse().expect("valid count")))
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
    // Test that Unicode character length is calculated correctly
    let output = Command::cargo_bin("word-tally")
        .expect("word-tally binary should be available")
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
        .map(|line| {
            line.split_whitespace()
                .next()
                .expect("line should have at least one word")
        })
        .collect();

    assert!(!words.contains(&"a")); // 1 char
    assert!(words.contains(&"café")); // 4 chars
    assert!(words.contains(&"naive")); // 5 chars
    assert!(words.contains(&"naïve")); // 5 chars
}

// =============================================================================
// ASCII/Unicode parity tests (for ASCII-only input)
// =============================================================================

#[test]
fn test_ascii_parity_basic() {
    test_encoding_parity_batch(&[
        ("hello world test data hello", &[]),
        ("don't can't it's we're I'll they'll", &[]),
        ("test123 456test hello2world 2fast2furious", &[]),
        ("hello, world! How are you? I'm fine... Really!", &[]),
    ]);
}

#[test]
fn test_case_handling_parity() {
    let input = "Hello WORLD hello World HELLO world";
    test_encoding_parity_batch(&[(input, &["--case=original"]), (input, &["--case=lower"])]);
}

#[test]
fn test_edge_cases_parity() {
    test_encoding_parity_batch(&[
        ("word", &[]),
        ("", &[]),
        ("   ", &[]),
        ("a b c", &[]),
        ("123 456", &[]),
        ("test's", &[]),
        ("well-known", &[]),
        ("C:\\path\\file", &[]),
        ("test...test", &[]),
        ("test--test", &[]),
    ]);
}

#[test]
fn test_multiple_apostrophes() {
    test_encoding_parity("'twas rock'n'roll y'all ma'am", &[]);
}

// =============================================================================
// Documented differences between ASCII and Unicode
// =============================================================================

#[test]
fn test_documented_differences() {
    // Test that Unicode handles non-ASCII characters while ASCII fails
    let unicode_output = word_tally()
        .arg("--encoding=unicode")
        .write_stdin("café naïve")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    word_tally()
        .arg("--encoding=ascii")
        .write_stdin("café naïve")
        .assert()
        .failure()
        .code(65); // ASCII mode fails on non-ASCII input

    let unicode_str = String::from_utf8_lossy(&unicode_output);

    // Unicode should successfully process non-ASCII characters
    assert!(unicode_str.contains("café"));
    assert!(unicode_str.contains("naïve"));
}

#[test]
fn test_ascii_simplicity() {
    let output = word_tally()
        .arg("--encoding=ascii")
        .arg("--case=lower")
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
        .map(|line| line.split_whitespace().next().expect("valid word"))
        .collect();

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
    for expected in expected_words {
        assert!(words.contains(&expected), "Missing word: {expected}");
    }
}
