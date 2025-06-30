//! Tests for CLI pattern matching.

use assert_cmd::Command;
use predicates::prelude::*;

fn word_tally() -> Command {
    Command::cargo_bin("word-tally").expect("process test")
}

fn test_patterns(args: &[&str], input: &str, include_words: &[&str], exclude_words: &[&str]) {
    let mut assertion = word_tally();
    for arg in args {
        assertion.arg(*arg);
    }

    let mut pred = assertion.write_stdin(input).assert().success();

    for word in include_words {
        pred = pred.stdout(predicate::str::contains(format!("{word} 1")));
    }

    for word in exclude_words {
        pred = pred.stdout(predicate::str::contains(*word).not());
    }
}

#[test]
fn exclude_patterns() {
    test_patterns(
        &["--exclude=^t.*"],
        "truth tomorrow sublime narrow",
        &["sublime", "narrow"],
        &["truth", "tomorrow"],
    );
}

#[test]
fn include_patterns() {
    test_patterns(
        &["--include=^t.*"],
        "truth tomorrow sublime narrow",
        &["truth", "tomorrow"],
        &["sublime", "narrow"],
    );
}

#[test]
fn multiple_exclude_patterns() {
    test_patterns(
        &["--exclude=^a.*", "--exclude=^b.*"],
        "angel beauty certain delight",
        &["certain", "delight"],
        &["angel", "beauty"],
    );
}

#[test]
fn multiple_include_patterns() {
    test_patterns(
        &["--include=^a.*", "--include=^b.*"],
        "angel beauty certain delight",
        &["angel", "beauty"],
        &["certain", "delight"],
    );
}

#[test]
fn combine_exclusions() {
    test_patterns(
        &["--exclude-words=hope,soul", "--exclude=^a.*"],
        "hope soul angel beauty certain",
        &["beauty", "certain"],
        &["hope", "soul", "angel"],
    );
}

#[test]
fn exclude_words_list() {
    test_patterns(
        &["--exclude-words=the,a,but"],
        "the narrow certain fame but a hope",
        &["narrow", "certain", "fame", "hope"],
        &["the", "but"],
    );
}
