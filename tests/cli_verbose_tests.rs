use assert_cmd::Command;
use predicates::str::contains;

fn word_tally() -> Command {
    Command::cargo_bin("word-tally").expect("process test")
}

#[test]
fn verbose_without_input() {
    word_tally()
        .arg("-v")
        .assert()
        .success()
        .stderr("source -\ntotal-words 0\nunique-words 0\ndelimiter \" \"\ncase lower\norder desc\nprocessing sequential\nio streamed\nmin-chars none\nmin-count none\nexclude-words none\nexclude-patterns none\ninclude-patterns none\n")
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
    let output = word_tally()
        .arg("-v")
        .arg("--format=json")
        .output()
        .expect("failed to execute process");

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Parse JSON and verify structure
    let json: serde_json::Value =
        serde_json::from_str(&stderr).expect("Failed to parse verbose JSON output");

    // Check all expected fields exist
    assert_eq!(json["source"], "-");
    assert_eq!(json["totalWords"], 0);
    assert_eq!(json["uniqueWords"], 0);
    assert_eq!(json["delimiter"], "\" \"");
    assert_eq!(json["case"], "lower");
    assert_eq!(json["order"], "desc");
    assert_eq!(json["processing"], "sequential");
    assert_eq!(json["io"], "streamed");
    assert_eq!(json["minChars"], serde_json::Value::Null);
    assert_eq!(json["minCount"], serde_json::Value::Null);
    assert_eq!(json["excludeWords"], serde_json::Value::Null);
    assert_eq!(json["excludePatterns"], serde_json::Value::Null);
    assert_eq!(json["includePatterns"], serde_json::Value::Null);
}

#[test]
fn verbose_json_with_options() {
    let output = word_tally()
        .arg("-v")
        .arg("--format=json")
        .arg("--min-chars=4")
        .arg("--min-count=2")
        .arg("--exclude-words=the,and")
        .write_stdin("the the and hope hope")
        .output()
        .expect("failed to execute process");

    let stderr = String::from_utf8_lossy(&output.stderr);
    let json: serde_json::Value =
        serde_json::from_str(&stderr).expect("Failed to parse verbose JSON output");

    // Check metrics after filtering
    assert_eq!(json["totalWords"], 2); // Only "hope hope"
    assert_eq!(json["uniqueWords"], 1); // Only "hope"
    assert_eq!(json["minChars"], 4);
    assert_eq!(json["minCount"], 2);
    assert_eq!(json["excludeWords"], serde_json::json!(["the", "and"]));
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
    let output = word_tally()
        .arg("-v")
        .arg("--format=csv")
        .output()
        .expect("failed to execute process");

    let stderr = String::from_utf8_lossy(&output.stderr);
    let lines: Vec<&str> = stderr.lines().collect();

    // Check CSV header is first line
    assert!(lines[0].contains("source"));
    assert!(lines[0].contains("total-words"));
    assert!(lines[0].contains("unique-words"));

    // Check that second line contains expected data
    assert!(lines[1].contains("-")); // source
    assert!(lines[1].contains("0")); // totalWords and uniqueWords
}

#[test]
fn verbose_csv_with_options() {
    let output = word_tally()
        .arg("-v")
        .arg("--format=csv")
        .arg("--min-chars=4")
        .arg("--min-count=2")
        .arg("--exclude-words=the,and")
        .write_stdin("the the and hope hope")
        .output()
        .expect("failed to execute process");

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Check CSV header contains expected fields
    assert!(stderr.contains("source"));
    assert!(stderr.contains("total-words"));
    assert!(stderr.contains("unique-words"));
    assert!(stderr.contains("min-chars"));
    assert!(stderr.contains("min-count"));
    assert!(stderr.contains("exclude-words"));

    // Check metrics in second line
    let lines: Vec<&str> = stderr.lines().collect();
    assert!(lines[1].contains("2")); // totalWords: Only "hope hope" remain after filters
    assert!(lines[1].contains("1")); // uniqueWords: Only "hope" is unique
    assert!(lines[1].contains("4")); // minChars
    assert!(lines[1].contains("the"));
    assert!(lines[1].contains("and"));
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
    let delimiter_index = headers.iter().position(|&h| h == "delimiter").unwrap();
    let values: Vec<&str> = lines[1].split(',').collect();

    assert_eq!(values[delimiter_index], r#"""" """"#);

    // Test with custom delimiter
    let output = word_tally()
        .arg("-v")
        .arg("--format=csv")
        .arg("--delimiter=|")
        .output()
        .expect("failed to execute process");

    let stderr = String::from_utf8_lossy(&output.stderr);
    let lines: Vec<&str> = stderr.lines().collect();
    let new_values: Vec<&str> = lines[1].split(',').collect();
    assert_eq!(new_values[delimiter_index], r#""""|""""#);
}
