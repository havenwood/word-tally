//! Tests for delimiter output formatting in the CLI.

use assert_cmd::Command;

fn word_tally() -> Command {
    Command::cargo_bin("word-tally").expect("find word-tally binary")
}

#[test]
fn test_backslash_output_delimiter() {
    // Test that escaped backslash can be used as output delimiter
    let output = word_tally()
        .arg("--field-delimiter=\\\\")
        .write_stdin("apple banana cherry apple banana")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8_lossy(&output);

    // Check that all expected entries are present (order may vary for equal counts)
    assert!(output_str.contains("apple\\2"));
    assert!(output_str.contains("banana\\2"));
    assert!(output_str.contains("cherry\\1"));

    // Verify format and count
    assert_eq!(output_str.trim().lines().count(), 3);
}

#[test]
fn test_double_escaped_backslash_output_delimiter() {
    // Test four backslashes produces double backslash in output
    let output = word_tally()
        .arg("--field-delimiter=\\\\\\\\")
        .arg("--sort=desc")
        .write_stdin("apple banana cherry apple banana")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8_lossy(&output);

    // Check the expected format for all entries
    assert!(output_str.contains("apple\\\\2"));
    assert!(output_str.contains("banana\\\\2"));
    assert!(output_str.contains("cherry\\\\1"));
}

#[test]
fn test_single_backslash_delimiter_succeeds() {
    // Test that single backslash is allowed as literal backslash
    let output = word_tally()
        .arg("--field-delimiter=\\")
        .write_stdin("apple banana")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8_lossy(&output);
    assert!(output_str.contains("banana\\1"));
    assert!(output_str.contains("apple\\1"));
}

#[test]
fn test_backslash_in_input_is_not_word_boundary() {
    // Verify that backslash in input is NOT treated as a word delimiter
    // word-tally uses Unicode word segmentation, not custom delimiters for input
    let output = word_tally()
        .arg("--sort=desc")
        .write_stdin("apple\\banana\\cherry\\apple\\banana")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8_lossy(&output);
    let lines: Vec<&str> = output_str.trim().lines().collect();

    // Check we have 3 entries
    assert_eq!(lines.len(), 3);

    // Check that cherry (count 1) is last
    assert_eq!(lines[2], "cherry 1");

    // Check that apple and banana both have count 2
    assert!(output_str.contains("apple 2"));
    assert!(output_str.contains("banana 2"));
}

#[test]
fn test_backslash_delimiter_with_backslash_in_input() {
    // Test combining backslash in input with backslash output delimiter
    let output = word_tally()
        .arg("--field-delimiter=\\\\")
        .arg("--sort=desc")
        .write_stdin("apple\\banana\\cherry\\apple\\banana")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8_lossy(&output);
    let lines: Vec<&str> = output_str.trim().lines().collect();

    // Check that we have 3 lines
    assert_eq!(lines.len(), 3);

    // Check that cherry\\1 is last (lowest count)
    assert_eq!(lines[2], "cherry\\1");

    // Check that apple and banana both have count 2 (order between them may vary)
    assert!(lines.contains(&"apple\\2"));
    assert!(lines.contains(&"banana\\2"));
}

#[test]
fn test_various_escape_sequences_as_delimiters() {
    // Test tab delimiter
    word_tally()
        .arg("--field-delimiter=\\t")
        .write_stdin("hello world hello")
        .assert()
        .success()
        .stdout("hello\t2\nworld\t1\n");

    // Test newline delimiter (creates double newlines in output)
    word_tally()
        .arg("--field-delimiter=\\n")
        .write_stdin("hello world hello")
        .assert()
        .success()
        .stdout("hello\n2\nworld\n1\n");
}

#[test]
fn test_delimiter_only_affects_output_not_input() {
    // Comprehensive test showing delimiter doesn't affect input parsing
    let input = "test1,test2;test3|test4\\test5:test6 test7\ttest8\ntest9";

    // Default delimiter (space)
    let expected_default = word_tally()
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    // Custom delimiter should produce same word counts, just different formatting
    let output_comma = word_tally()
        .arg("--field-delimiter=,")
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    // Parse outputs to verify same words were found
    let parse_output = |output: &[u8]| -> Vec<(String, u64)> {
        String::from_utf8_lossy(output)
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.splitn(2, [' ', ',']).collect();
                if parts.len() == 2 {
                    parts[1]
                        .parse::<u64>()
                        .ok()
                        .map(|count| (parts[0].to_string(), count))
                } else {
                    None
                }
            })
            .collect()
    };

    let words_default = parse_output(&expected_default);
    let words_comma = parse_output(&output_comma);

    // Same number of unique words found
    assert_eq!(words_default.len(), words_comma.len());

    // Same words with same counts (order might differ)
    for (word, count) in &words_default {
        assert!(
            words_comma.iter().any(|(w, c)| w == word && c == count),
            "Word '{word}' with count {count} not found in comma-delimited output"
        );
    }
}

#[test]
fn test_custom_record_delimiter() {
    // Test custom record delimiter (semicolon)
    word_tally()
        .arg("--entry-delimiter=;")
        .write_stdin("hello world hello")
        .assert()
        .success()
        .stdout("hello 2;world 1;");

    // Test with both custom field and record delimiters
    word_tally()
        .arg("--field-delimiter=,")
        .arg("--entry-delimiter=|")
        .write_stdin("apple banana apple")
        .assert()
        .success()
        .stdout("apple,2|banana,1|");
}

#[test]
fn test_escaped_record_delimiter() {
    // Test escaped tab as record delimiter
    word_tally()
        .arg("--entry-delimiter=\\t")
        .write_stdin("hello world hello")
        .assert()
        .success()
        .stdout("hello 2\tworld 1\t");

    // Test double escaped backslash as record delimiter
    let output = word_tally()
        .arg("--entry-delimiter=\\\\")
        .write_stdin("apple banana")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8_lossy(&output);
    assert!(output_str.contains("apple 1\\"));
    assert!(output_str.contains("banana 1\\"));
}
