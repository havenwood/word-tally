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
fn verbose_with_csv_format() {
    word_tally()
        .arg("-v")
        .arg("--format=csv")
        .write_stdin("narrow fame")
        .assert()
        .success()
        .stderr(contains("total-words,2"))
        .stderr(contains("unique-words,2"));
}
