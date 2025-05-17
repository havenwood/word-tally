use assert_cmd::Command;

fn word_tally() -> Command {
    Command::cargo_bin("word-tally").unwrap()
}

#[test]
fn exclude_patterns() {
    word_tally()
        .arg("--exclude=^t.*")
        .write_stdin("truth tomorrow sublime narrow")
        .assert()
        .success()
        .stdout("sublime 1\nnarrow 1\n");
}

#[test]
fn include_patterns() {
    word_tally()
        .arg("--include=^t.*")
        .write_stdin("truth tomorrow sublime narrow")
        .assert()
        .success()
        .stdout("truth 1\ntomorrow 1\n");
}

#[test]
fn multiple_exclude_patterns() {
    word_tally()
        .arg("--exclude=^a.*")
        .arg("--exclude=^b.*")
        .write_stdin("angel beauty certain delight")
        .assert()
        .success()
        .stdout("certain 1\ndelight 1\n");
}

#[test]
fn multiple_include_patterns() {
    word_tally()
        .arg("--include=^a.*")
        .arg("--include=^b.*")
        .write_stdin("angel beauty certain delight")
        .assert()
        .success()
        .stdout("angel 1\nbeauty 1\n");
}

#[test]
fn combine_exclusions() {
    word_tally()
        .arg("--exclude-words=hope,soul")
        .arg("--exclude=^a.*")
        .write_stdin("hope soul angel beauty certain")
        .assert()
        .success()
        .stdout("beauty 1\ncertain 1\n");
}

#[test]
fn exclude_words_list() {
    word_tally()
        .arg("--exclude-words=the,a,but")
        .write_stdin("the narrow certain fame but a hope")
        .assert()
        .success()
        .stdout("narrow 1\ncertain 1\nfame 1\nhope 1\n");
}
