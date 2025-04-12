use assert_cmd::Command;
use predicates::prelude::PredicateBooleanExt;
use predicates::str::{self, contains};
use std::fs;
fn word_tally() -> Command {
    Command::cargo_bin("word-tally").unwrap()
}

#[test]
fn version() {
    let assert = word_tally().arg("-V").assert();
    assert.success().stdout(str::starts_with("word-tally "));
}

#[test]
fn help() {
    let assert = word_tally().arg("-h").assert();
    assert.success().stdout(str::contains("\nUsage"));
}

#[test]
fn verbose_without_input() {
    let assert = word_tally().arg("-v").assert();
    assert
        .success()
        .stderr("source -\ntotal-words 0\nunique-words 0\ndelimiter \" \"\ncase lower\norder desc\nmin-chars none\nmin-count none\nexclude-words none\n")
        .stdout("");
}

#[test]
fn verbose_with_min_chars() {
    let assert = word_tally().arg("-v").arg("--min-chars=42").assert();
    assert
        .success()
        .stderr("source -\ntotal-words 0\nunique-words 0\ndelimiter \" \"\ncase lower\norder desc\nmin-chars 42\nmin-count none\nexclude-words none\n")
        .stdout("");
}

#[test]
fn verbose_with_min_count() {
    let assert = word_tally().arg("-v").arg("--min-count=42").assert();
    assert
        .success()
        .stderr("source -\ntotal-words 0\nunique-words 0\ndelimiter \" \"\ncase lower\norder desc\nmin-chars none\nmin-count 42\nexclude-words none\n")
        .stdout("");
}

#[test]
fn verbose_with_exclude() {
    let assert = word_tally()
        .arg("-v")
        .arg("--exclude=wombat,trees")
        .assert();
    assert
        .success()
        .stderr("source -\ntotal-words 0\nunique-words 0\ndelimiter \" \"\ncase lower\norder desc\nmin-chars none\nmin-count none\nexclude-words wombat,trees\n")
        .stdout("");
}

#[test]
fn verbose_with_input() {
    let assert = word_tally().write_stdin("wombat").arg("-v").assert();
    assert
        .success()
        .stderr("source -\ntotal-words 1\nunique-words 1\ndelimiter \" \"\ncase lower\norder desc\nmin-chars none\nmin-count none\nexclude-words none\n\n")
        .stdout("wombat 1\n");
}

#[test]
fn output() {
    let assert = word_tally()
        .write_stdin("wombat")
        .arg("--output=test.txt")
        .assert();
    assert.success().stdout("");
    assert_eq!("wombat 1\n", fs::read_to_string("test.txt").unwrap());
    fs::remove_file("test.txt").unwrap();
}

#[test]
fn delimiter() {
    let assert = word_tally().write_stdin("wombat").arg("-d\t").assert();
    assert.success().stdout("wombat\t1\n");
}

#[test]
fn case_default() {
    let assert = word_tally().write_stdin("wOmBaT TrEeS trees").assert();
    assert.success().stdout("trees 2\nwombat 1\n");
}

#[test]
fn case_upper() {
    let assert = word_tally()
        .write_stdin("wOmBaT TrEeS wombat")
        .arg("--case=upper")
        .assert();
    assert.success().stdout("WOMBAT 2\nTREES 1\n");
}

#[test]
fn case_original() {
    let assert = word_tally()
        .write_stdin("wombat wOmBaT wombat wOmBaT wombat TrEeS")
        .arg("--case=original")
        .assert();
    assert.success().stdout("wombat 3\nwOmBaT 2\nTrEeS 1\n");
}

#[test]
fn sort_default() {
    let assert = word_tally().write_stdin("bb a bb a ccc a").assert();
    assert.success().stdout("a 3\nbb 2\nccc 1\n");
}

#[test]
fn sort_asc() {
    let assert = word_tally()
        .write_stdin("bb a bb a ccc a")
        .arg("--sort=asc")
        .assert();
    assert.success().stdout("ccc 1\nbb 2\na 3\n");
}

#[test]
fn no_words() {
    let assert = word_tally().write_stdin("").assert();
    assert.success().stdout("");
}

#[test]
fn test_discard_words() {
    let input = "The tree that would grow to heaven must send its roots to hell.";
    let mut cmd = Command::cargo_bin("word-tally").unwrap();
    cmd.write_stdin(input)
        .arg("--exclude=heaven,hell")
        .assert()
        .success()
        .stdout(contains("tree").and(contains("heaven").not().and(contains("hell").not())));
}

#[test]
fn format_json() {
    let assert = word_tally()
        .write_stdin("wombat wombat bat")
        .arg("--format=json")
        .assert();
    assert.success().stdout(contains("[\"wombat\",2]").and(contains("[\"bat\",1]")));
}

#[test]
fn format_csv() {
    let assert = word_tally()
        .write_stdin("wombat wombat bat")
        .arg("--format=csv")
        .assert();
    assert.success()
        .stdout(contains("word,count"))
        .stdout(contains("wombat,2"))
        .stdout(contains("bat,1"));
}

#[test]
fn stdin_with_parallel() {
    // Test with a small input to verify the fix for the zero chunk size issue
    let assert = word_tally()
        .write_stdin("hello world")
        .arg("--parallel")
        .assert();
    assert.success()
        .stdout(contains("hello 1"))
        .stdout(contains("world 1"));

    // Test with a multi-line input
    let assert = word_tally()
        .write_stdin("hello world\ngoodbye universe\nhello again")
        .arg("--parallel")
        .assert();
    assert.success()
        .stdout(contains("hello 2"))
        .stdout(contains("world 1"))
        .stdout(contains("goodbye 1"))
        .stdout(contains("universe 1"))
        .stdout(contains("again 1"));
}

#[test]
fn parallel_with_env_vars() {
    let assert = word_tally()
        .env("WORD_TALLY_CHUNK_SIZE", "4096")
        .env("WORD_TALLY_THREADS", "2")
        .write_stdin("test environment variables with CLI")
        .arg("--parallel")
        .assert();

    assert.success()
        .stdout(contains("test 1"))
        .stdout(contains("environment 1"))
        .stdout(contains("variables 1"))
        .stdout(contains("with 1"))
        .stdout(contains("cli 1"));
}

#[test]
fn parallel_with_large_chunk() {
    let assert = word_tally()
        .env("WORD_TALLY_CHUNK_SIZE", "65536")
        .write_stdin("test with very large chunk size")
        .arg("--parallel")
        .assert();

    assert.success()
        .stdout(contains("test 1"))
        .stdout(contains("with 1"))
        .stdout(contains("very 1"))
        .stdout(contains("large 1"))
        .stdout(contains("chunk 1"))
        .stdout(contains("size 1"));
}
