use assert_cmd::Command;
use predicates::prelude::*;

fn word_tally() -> Command {
    Command::cargo_bin("word-tally").expect("process test")
}

#[test]
fn exclude_patterns() {
    word_tally()
        .arg("--exclude=^t.*")
        .write_stdin("truth tomorrow sublime narrow")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("sublime 1")
                .and(predicate::str::contains("narrow 1"))
                .and(predicate::str::contains("truth").not())
                .and(predicate::str::contains("tomorrow").not()),
        );
}

#[test]
fn include_patterns() {
    word_tally()
        .arg("--include=^t.*")
        .write_stdin("truth tomorrow sublime narrow")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("truth 1")
                .and(predicate::str::contains("tomorrow 1"))
                .and(predicate::str::contains("sublime").not())
                .and(predicate::str::contains("narrow").not()),
        );
}

#[test]
fn multiple_exclude_patterns() {
    word_tally()
        .arg("--exclude=^a.*")
        .arg("--exclude=^b.*")
        .write_stdin("angel beauty certain delight")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("certain 1")
                .and(predicate::str::contains("delight 1"))
                .and(predicate::str::contains("angel").not())
                .and(predicate::str::contains("beauty").not()),
        );
}

#[test]
fn multiple_include_patterns() {
    word_tally()
        .arg("--include=^a.*")
        .arg("--include=^b.*")
        .write_stdin("angel beauty certain delight")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("angel 1")
                .and(predicate::str::contains("beauty 1"))
                .and(predicate::str::contains("certain").not())
                .and(predicate::str::contains("delight").not()),
        );
}

#[test]
fn combine_exclusions() {
    word_tally()
        .arg("--exclude-words=hope,soul")
        .arg("--exclude=^a.*")
        .write_stdin("hope soul angel beauty certain")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("beauty 1")
                .and(predicate::str::contains("certain 1"))
                .and(predicate::str::contains("hope").not())
                .and(predicate::str::contains("soul").not())
                .and(predicate::str::contains("angel").not()),
        );
}

#[test]
fn exclude_words_list() {
    word_tally()
        .arg("--exclude-words=the,a,but")
        .write_stdin("the narrow certain fame but a hope")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("narrow 1").and(
                predicate::str::contains("certain 1")
                    .and(predicate::str::contains("fame 1"))
                    .and(predicate::str::contains("hope 1")),
            ),
        );
}
