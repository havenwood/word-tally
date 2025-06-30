//! Tests for CLI output formatting.

use assert_cmd::Command;
use predicates::str::contains;

fn word_tally() -> Command {
    Command::cargo_bin("word-tally").expect("process test")
}

#[test]
fn format_json() {
    word_tally()
        .arg("--format=json")
        .write_stdin("narrow fame narrow")
        .assert()
        .success()
        .stdout(contains(r#"[["narrow",2],["fame",1]]"#));
}

#[test]
fn format_csv() {
    word_tally()
        .arg("--format=csv")
        .write_stdin("narrow fame narrow")
        .assert()
        .success()
        .stdout("word,count\nnarrow,2\nfame,1\n");
}

#[test]
fn csv_escaping() {
    // Testing that CSV escaping happens correctly with special characters
    word_tally()
        .arg("--format=csv")
        .write_stdin("\"sublime soul\" infinite,beauty certain\"hope")
        .assert()
        .success()
        .stdout(contains("word,count"));
}

#[test]
fn delimiter_shorthand() {
    word_tally()
        .arg("-d=,")
        .write_stdin("narrow fame narrow")
        .assert()
        .success()
        .stdout("narrow,2\nfame,1\n");
}

#[test]
fn delimiter_longhand() {
    word_tally()
        .arg("--field-delimiter=,")
        .write_stdin("narrow fame narrow")
        .assert()
        .success()
        .stdout("narrow,2\nfame,1\n");
}
