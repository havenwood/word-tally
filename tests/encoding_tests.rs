//! Tests for encoding functionality (ASCII and Unicode modes)

use assert_cmd::Command;
use std::collections::HashSet;
use word_tally::{Case, TallyMap};

/// Parse word-tally output into a set of (word, count) pairs
fn parse_output(output: &[u8]) -> HashSet<(String, usize)> {
    std::str::from_utf8(output)
        .unwrap()
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            assert_eq!(parts.len(), 2, "Invalid output line: {line}");
            (parts[0].to_string(), parts[1].parse::<usize>().unwrap())
        })
        .collect()
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
    let input_text = "café über señor hello world café";

    let output = Command::cargo_bin("word-tally")
        .unwrap()
        .arg("--encoding=unicode")
        .arg("--case=lower")
        .arg("--sort=desc")
        .write_stdin(input_text)
        .output()
        .expect("Failed to execute word-tally");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();

    // Count occurrences
    let mut word_counts = std::collections::HashMap::new();
    for line in &lines {
        if let Some((word, count)) = line.split_once(char::is_whitespace) {
            let count: usize = count.trim().parse().unwrap();
            word_counts.insert(word.trim(), count);
        }
    }

    assert_eq!(word_counts.get("café"), Some(&2));
    assert_eq!(word_counts.get("über"), Some(&1));
    assert_eq!(word_counts.get("señor"), Some(&1));
    assert_eq!(word_counts.get("hello"), Some(&1));
    assert_eq!(word_counts.get("world"), Some(&1));
}

#[test]
fn test_unicode_min_chars() {
    // Test that Unicode character length is calculated correctly
    let output = Command::cargo_bin("word-tally")
        .unwrap()
        .arg("--min-chars=4")
        .write_stdin("a café naive naïve")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let words: Vec<&str> = std::str::from_utf8(&output)
        .unwrap()
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| line.split_whitespace().next().unwrap())
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
fn test_basic_ascii_parity() {
    let input = "hello world test data hello";

    let unicode_output = Command::cargo_bin("word-tally")
        .unwrap()
        .arg("--encoding=unicode")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let ascii_output = Command::cargo_bin("word-tally")
        .unwrap()
        .arg("--encoding=ascii")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let unicode_words = parse_output(&unicode_output);
    let ascii_words = parse_output(&ascii_output);

    assert_eq!(unicode_words, ascii_words, "Word counts should match");
}

#[test]
fn test_contractions_parity() {
    let input = "don't can't it's we're I'll they'll";

    let unicode_output = Command::cargo_bin("word-tally")
        .unwrap()
        .arg("--encoding=unicode")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let ascii_output = Command::cargo_bin("word-tally")
        .unwrap()
        .arg("--encoding=ascii")
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
        "Contractions should be handled identically"
    );
}

#[test]
fn test_alphanumeric_parity() {
    let input = "test123 456test hello2world 2fast2furious";

    let unicode_output = Command::cargo_bin("word-tally")
        .unwrap()
        .arg("--encoding=unicode")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let ascii_output = Command::cargo_bin("word-tally")
        .unwrap()
        .arg("--encoding=ascii")
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
        "Alphanumeric words should match"
    );
}

#[test]
fn test_punctuation_parity() {
    let input = "hello, world! How are you? I'm fine... Really!";

    let unicode_output = Command::cargo_bin("word-tally")
        .unwrap()
        .arg("--encoding=unicode")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let ascii_output = Command::cargo_bin("word-tally")
        .unwrap()
        .arg("--encoding=ascii")
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
        "Punctuation handling should match"
    );
}

#[test]
fn test_case_handling_parity() {
    let input = "Hello WORLD hello World HELLO world";

    // Test with original case
    let unicode_output = Command::cargo_bin("word-tally")
        .unwrap()
        .arg("--encoding=unicode")
        .arg("--case=original")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let ascii_output = Command::cargo_bin("word-tally")
        .unwrap()
        .arg("--encoding=ascii")
        .arg("--case=original")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    assert_eq!(
        parse_output(&unicode_output),
        parse_output(&ascii_output),
        "Original case handling should match"
    );

    // Test with lowercase
    let unicode_output = Command::cargo_bin("word-tally")
        .unwrap()
        .arg("--encoding=unicode")
        .arg("--case=lower")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let ascii_output = Command::cargo_bin("word-tally")
        .unwrap()
        .arg("--encoding=ascii")
        .arg("--case=lower")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    assert_eq!(
        parse_output(&unicode_output),
        parse_output(&ascii_output),
        "Lowercase handling should match"
    );
}

#[test]
fn test_edge_cases_parity() {
    // Test various edge cases
    let test_cases = vec![
        "word",           // single word
        "",               // empty input
        "   ",            // only whitespace
        "a b c",          // single letters
        "123 456",        // only numbers
        "test's",         // possessive
        "well-known",     // hyphenated (should be split)
        "C:\\path\\file", // path-like
        "test...test",    // ellipsis
        "test--test",     // double dash
    ];

    for input in test_cases {
        let unicode_output = Command::cargo_bin("word-tally")
            .unwrap()
            .arg("--encoding=unicode")
            .write_stdin(input)
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();

        let ascii_output = Command::cargo_bin("word-tally")
            .unwrap()
            .arg("--encoding=ascii")
            .write_stdin(input)
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();

        assert_eq!(
            parse_output(&unicode_output),
            parse_output(&ascii_output),
            "Edge case '{input}' should be handled identically"
        );
    }
}

#[test]
fn test_multiple_apostrophes() {
    // Special focus on apostrophe handling
    let input = "'twas rock'n'roll y'all ma'am";

    let unicode_output = Command::cargo_bin("word-tally")
        .unwrap()
        .arg("--encoding=unicode")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let ascii_output = Command::cargo_bin("word-tally")
        .unwrap()
        .arg("--encoding=ascii")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let unicode_words = parse_output(&unicode_output);
    let ascii_words = parse_output(&ascii_output);

    // Print for debugging if they don't match
    if unicode_words != ascii_words {
        eprintln!("Unicode: {unicode_words:?}");
        eprintln!("ASCII: {ascii_words:?}");
    }

    assert_eq!(
        unicode_words, ascii_words,
        "Apostrophe handling should match"
    );
}

// =============================================================================
// Documented differences between ASCII and Unicode
// =============================================================================

#[test]
fn test_documented_differences() {
    // These are cases where ASCII and Unicode intentionally differ
    let test_cases = vec![
        // (input, unicode_words, ascii_words, explanation)
        (
            "'hello'",
            vec!["hello"],
            vec!["hello'"],
            "Unicode strips quotes, ASCII keeps trailing apostrophe",
        ),
        (
            "email@test.com",
            vec!["email", "test.com"],
            vec!["email", "test", "com"],
            "Unicode handles email-like patterns, ASCII splits on @",
        ),
        (
            "$100.50",
            vec!["100.50"],
            vec!["100", "50"],
            "Unicode keeps decimal numbers together, ASCII splits on period",
        ),
        (
            "3.14159",
            vec!["3.14159"],
            vec!["3", "14159"],
            "Unicode handles decimals, ASCII splits on period",
        ),
        (
            "test--test",
            vec!["test"], // Count will be 2, but only one unique word
            vec!["test"], // Count will be 2, but only one unique word
            "Both split on double dash",
        ),
        (
            "C:\\path\\file",
            vec!["C", "path", "file"],
            vec!["C", "path", "file"],
            "Both split on backslashes",
        ),
    ];

    for (input, expected_unicode, expected_ascii, explanation) in test_cases {
        println!("Testing: {input} - {explanation}");

        // Get Unicode output
        let unicode_output = Command::cargo_bin("word-tally")
            .unwrap()
            .arg("--encoding=unicode")
            .write_stdin(input)
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();

        let unicode_words: Vec<String> = std::str::from_utf8(&unicode_output)
            .unwrap()
            .lines()
            .filter(|line| !line.is_empty())
            .map(|line| line.split_whitespace().next().unwrap().to_string())
            .collect();

        // Get ASCII output
        let ascii_output = Command::cargo_bin("word-tally")
            .unwrap()
            .arg("--encoding=ascii")
            .write_stdin(input)
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();

        let ascii_words: Vec<String> = std::str::from_utf8(&ascii_output)
            .unwrap()
            .lines()
            .filter(|line| !line.is_empty())
            .map(|line| line.split_whitespace().next().unwrap().to_string())
            .collect();

        // Sort for comparison since order may differ
        let mut unicode_sorted = unicode_words.clone();
        unicode_sorted.sort();
        let mut expected_unicode_sorted = expected_unicode
            .iter()
            .map(|s| (*s).to_string())
            .collect::<Vec<_>>();
        expected_unicode_sorted.sort();

        let mut ascii_sorted = ascii_words.clone();
        ascii_sorted.sort();
        let mut expected_ascii_sorted = expected_ascii
            .iter()
            .map(|s| (*s).to_string())
            .collect::<Vec<_>>();
        expected_ascii_sorted.sort();

        assert_eq!(
            unicode_sorted, expected_unicode_sorted,
            "Unicode output for '{input}' doesn't match expected"
        );
        assert_eq!(
            ascii_sorted, expected_ascii_sorted,
            "ASCII output for '{input}' doesn't match expected"
        );
    }
}

#[test]
fn test_ascii_simplicity() {
    // ASCII mode is intentionally simpler than Unicode
    // It only recognizes alphanumeric characters and apostrophes as word characters

    let input = "test@example.com, hello-world, 3.14, 'quoted', test's, rock'n'roll";

    let output = Command::cargo_bin("word-tally")
        .unwrap()
        .arg("--encoding=ascii")
        .arg("--case=lower")
        .arg("--sort=asc")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let words: Vec<&str> = std::str::from_utf8(&output)
        .unwrap()
        .lines()
        .filter(|line| !line.is_empty())
        .map(|line| line.split_whitespace().next().unwrap())
        .collect();

    // ASCII splits on all non-alphanumeric except apostrophes
    assert!(words.contains(&"test"));
    assert!(words.contains(&"example"));
    assert!(words.contains(&"com"));
    assert!(words.contains(&"hello"));
    assert!(words.contains(&"world"));
    assert!(words.contains(&"3"));
    assert!(words.contains(&"14"));
    assert!(words.contains(&"quoted'")); // Note: keeps trailing apostrophe
    assert!(words.contains(&"test's"));
    assert!(words.contains(&"rock'n'roll"));
}
