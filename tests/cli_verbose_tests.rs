//! Tests for CLI verbose output.

use assert_cmd::Command;
use predicates::str::contains;

fn word_tally() -> Command {
    Command::cargo_bin("word-tally").expect("process test")
}

fn test_verbose_json(args: &[&str], stdin: Option<&str>, checks: &[(&str, serde_json::Value)]) {
    let mut binding = word_tally();
    let cmd = binding.arg("-v").arg("--format=json");
    for arg in args {
        cmd.arg(*arg);
    }

    let output = if let Some(input) = stdin {
        cmd.write_stdin(input).output().expect("process execution")
    } else {
        cmd.output().expect("process execution")
    };

    let stderr = String::from_utf8_lossy(&output.stderr);
    let json: serde_json::Value = serde_json::from_str(&stderr).expect("parse json");

    for (key, expected) in checks {
        assert_eq!(json[key], *expected, "JSON field {key} mismatch");
    }
}

fn test_verbose_csv(args: &[&str], stdin: Option<&str>, expected_patterns: &[&str]) {
    let mut binding = word_tally();
    let cmd = binding.arg("-v").arg("--format=csv");
    for arg in args {
        cmd.arg(*arg);
    }

    let output = if let Some(input) = stdin {
        cmd.write_stdin(input).output().expect("process execution")
    } else {
        cmd.output().expect("process execution")
    };

    let stderr = String::from_utf8_lossy(&output.stderr);
    for pattern in expected_patterns {
        assert!(
            stderr.contains(pattern),
            "CSV output missing pattern: {pattern}"
        );
    }
}

#[test]
fn verbose_without_input() {
    word_tally()
        .arg("-v")
        .assert()
        .success()
        .stderr("source -\ntotal-words 0\nunique-words 0\ndelimiter \" \"\nentry-delimiter \"\\n\"\ncase original\norder desc\nio parallel-stream\nmin-chars none\nmin-count none\nexclude-words none\nexclude-patterns none\ninclude-patterns none\n")
        .stdout("");
}

#[test]
fn verbose_with_min_chars() {
    word_tally()
        .arg("-v")
        .arg("--min-chars=42")
        .assert()
        .success()
        .stderr(contains("min-chars 42"))
        .stdout("");
}

#[test]
fn verbose_with_min_count() {
    word_tally()
        .arg("-v")
        .arg("--min-count=42")
        .assert()
        .success()
        .stderr(contains("min-count 42"))
        .stdout("");
}

#[test]
fn verbose_with_exclude_words() {
    word_tally()
        .arg("-v")
        .arg("--exclude-words=hope,soul")
        .assert()
        .success()
        .stderr(contains("exclude-words hope,soul"))
        .stdout("");
}

#[test]
fn verbose_with_json_format() {
    word_tally()
        .arg("-v")
        .arg("--format=json")
        .write_stdin("narrow fame")
        .assert()
        .success()
        .stderr(contains("\"totalWords\":2"))
        .stderr(contains("\"uniqueWords\":2"));
}

#[test]
fn verbose_json_full_output() {
    test_verbose_json(
        &[],
        None,
        &[
            ("source", serde_json::json!("-")),
            ("totalWords", serde_json::json!(0)),
            ("uniqueWords", serde_json::json!(0)),
            ("fieldDelimiter", serde_json::json!("n/a")),
            ("entryDelimiter", serde_json::json!("n/a")),
            ("case", serde_json::json!("original")),
            ("order", serde_json::json!("desc")),
            ("io", serde_json::json!("parallel-stream")),
            ("minChars", serde_json::Value::Null),
            ("minCount", serde_json::Value::Null),
            ("excludeWords", serde_json::Value::Null),
            ("excludePatterns", serde_json::Value::Null),
            ("includePatterns", serde_json::Value::Null),
        ],
    );
}

#[test]
fn verbose_json_with_options() {
    test_verbose_json(
        &["--min-chars=4", "--min-count=2", "--exclude-words=the,and"],
        Some("the the and hope hope"),
        &[
            ("totalWords", serde_json::json!(2)),
            ("uniqueWords", serde_json::json!(1)),
            ("minChars", serde_json::json!(4)),
            ("minCount", serde_json::json!(2)),
            ("excludeWords", serde_json::json!(["the", "and"])),
        ],
    );
}

#[test]
fn verbose_with_csv_format() {
    word_tally()
        .arg("-v")
        .arg("--format=csv")
        .write_stdin("narrow fame")
        .assert()
        .success()
        .stderr(contains("total-words"))
        .stderr(contains("unique-words"))
        .stderr(contains("2,2")); // Both counts are 2
}

#[test]
fn verbose_csv_full_output() {
    test_verbose_csv(
        &[],
        None,
        &["source", "total-words", "unique-words", "-", "0"],
    );
}

#[test]
fn verbose_csv_with_options() {
    test_verbose_csv(
        &["--min-chars=4", "--min-count=2", "--exclude-words=the,and"],
        Some("the the and hope hope"),
        &[
            "source",
            "total-words",
            "unique-words",
            "min-chars",
            "min-count",
            "exclude-words",
            "2",
            "1",
            "4",
            "the",
            "and",
        ],
    );
}

#[test]
fn verbose_csv_delimiter_formatting() {
    // Test with default space delimiter
    let output = word_tally()
        .arg("-v")
        .arg("--format=csv")
        .output()
        .expect("failed to execute process");

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Check that space delimiter is nicely formatted
    assert!(stderr.contains("delimiter"));
    // Should show a space character, not quoted
    let lines: Vec<&str> = stderr.lines().collect();
    // Find the delimiter column
    let headers: Vec<&str> = lines[0].split(',').collect();
    let delimiter_index = headers
        .iter()
        .position(|&h| h == "delimiter")
        .expect("csv should have delimiter column");
    let values: Vec<&str> = lines[1].split(',').collect();

    assert_eq!(values[delimiter_index], "n/a");

    // Test that delimiters with non-text formats are rejected
    let output = word_tally()
        .arg("-v")
        .arg("--format=csv")
        .arg("--field-delimiter=|")
        .output()
        .expect("failed to execute process");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--field-delimiter and --entry-delimiter only apply to text format"));
}
