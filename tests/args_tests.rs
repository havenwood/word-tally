use assert_cmd::Command;
use predicates::str::contains;

#[test]
fn test_get_options() {
    let assert = Command::cargo_bin("word-tally")
        .unwrap()
        .arg("--case=upper")
        .arg("--sort=asc")
        .arg("--format=json")
        .arg("--min-chars=3")
        .arg("--parallel")
        .arg("-v")
        .write_stdin("hello world")
        .assert();

    assert
        .success()
        .stderr(contains("\"case\":\"upper\""))
        .stderr(contains("\"order\":\"asc\""))
        .stderr(contains("\"min-chars\":\"3\""));
}

#[test]
fn test_get_performance() {
    let assert = Command::cargo_bin("word-tally")
        .unwrap()
        .arg("--parallel")
        .arg("-v")
        .write_stdin("hello world")
        .assert();

    assert.success();
}

#[test]
fn test_io_shorthand_flag() {
    let assert = Command::cargo_bin("word-tally")
        .unwrap()
        .arg("-I=buffered")
        .arg("-v")
        .write_stdin("hello world")
        .assert();

    assert
        .success()
        .stderr(contains("io buffered"));
}

#[test]
fn test_get_formatting() {
    let assert = Command::cargo_bin("word-tally")
        .unwrap()
        .arg("--case=upper")
        .arg("--sort=asc")
        .arg("--format=json")
        .arg("-v")
        .write_stdin("hello")
        .assert();

    assert
        .success()
        .stderr(contains("\"case\":\"upper\""))
        .stderr(contains("\"order\":\"asc\""));
}

#[test]
fn test_get_filters() {
    let assert = Command::cargo_bin("word-tally")
        .unwrap()
        .arg("--min-chars=3")
        .arg("--min-count=2")
        .arg("--exclude-words=hello,world")
        .arg("-v")
        .write_stdin("hello world")
        .assert();

    assert
        .success()
        .stderr(contains("min-chars 3"))
        .stderr(contains("min-count 2"))
        .stderr(contains("exclude-words hello,world"));
}

#[test]
fn test_get_filters_with_patterns() {
    let assert = Command::cargo_bin("word-tally")
        .unwrap()
        .arg("--include=^h")
        .arg("--exclude=\\d+")
        .arg("-v")
        .write_stdin("hello 123 world")
        .assert();

    assert.success().stderr(contains("exclude-patterns \\d+"));
}
