//! Tests for CLI functionality.

use std::fs;

use assert_cmd::Command;
use predicates::{
    prelude::PredicateBooleanExt,
    str::{self, contains},
};
use tempfile::NamedTempFile;

fn word_tally() -> Command {
    Command::cargo_bin("word-tally").expect("process test")
}

fn create_temp_files_with_content(contents: &[&str]) -> Vec<NamedTempFile> {
    contents
        .iter()
        .map(|content| {
            let temp_file = NamedTempFile::new().expect("create temp file");
            fs::write(&temp_file, content).expect("write test file");
            temp_file
        })
        .collect()
}

fn test_word_counts(mut cmd: Command, expected_words: &[(&str, usize)]) {
    let mut assertion = cmd.assert().success();
    for (word, count) in expected_words {
        assertion = assertion.stdout(contains(format!("{word} {count}")));
    }
}

#[test]
fn version() {
    let assert = word_tally().arg("-V").assert();
    assert.success().stdout(str::starts_with("word-tally "));
}

#[test]
fn help() {
    let assert = word_tally().arg("-h").assert();
    assert.success().stdout(contains("\nUsage"));
}

#[test]
fn help_long() {
    let assert = word_tally().arg("--help").assert();
    assert.success().stdout(contains("\nUsage"));
}

#[test]
fn output_longhand() {
    let temp_file = NamedTempFile::new().expect("process test");
    let temp_path = temp_file.path().to_str().expect("process test");

    let assert = word_tally()
        .write_stdin("narrow")
        .arg(format!("--output={temp_path}"))
        .assert();
    assert.success().stdout("");
    assert_eq!(
        "narrow 1\n",
        fs::read_to_string(temp_path).expect("process test")
    );
}

#[test]
fn output_shorthand() {
    let temp_file = NamedTempFile::new().expect("process test");
    let temp_path = temp_file.path().to_str().expect("process test");

    let assert = word_tally()
        .write_stdin("narrow")
        .arg(format!("-o={temp_path}"))
        .assert();
    assert.success().stdout("");
    assert_eq!(
        "narrow 1\n",
        fs::read_to_string(temp_path).expect("process test")
    );
}

#[test]
fn case_default() {
    let assert = word_tally().write_stdin("nArRoW CeRtAiN certain").assert();
    assert
        .success()
        .stdout(contains("certain 1"))
        .stdout(contains("nArRoW 1"))
        .stdout(contains("CeRtAiN 1"));
}

#[test]
fn case_upper() {
    let assert = word_tally()
        .write_stdin("nArRoW CeRtAiN narrow")
        .arg("--case=upper")
        .assert();
    assert.success().stdout("NARROW 2\nCERTAIN 1\n");
}

#[test]
fn case_original() {
    let assert = word_tally()
        .write_stdin("narrow nArRoW narrow nArRoW narrow CeRtAiN")
        .arg("--case=original")
        .assert();
    assert.success().stdout("narrow 3\nnArRoW 2\nCeRtAiN 1\n");
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
    let input = "Hope is the thing with feathers that perches in the soul.";
    let mut cmd = word_tally();
    cmd.write_stdin(input)
        .arg("--exclude-words=feathers,soul")
        .assert()
        .success()
        .stdout(contains("Hope").and(contains("feathers").not().and(contains("soul").not())));
}

#[test]
fn stdin_with_parallel() {
    // Test with a small input
    let assert = word_tally().write_stdin("hope forever").assert();
    assert
        .success()
        .stdout(contains("hope 1"))
        .stdout(contains("forever 1"));

    // Test with a multi-line input
    let assert = word_tally()
        .write_stdin("hope forever\ninfinite beauty\nhope sublime")
        .assert();
    assert
        .success()
        .stdout(contains("hope 2"))
        .stdout(contains("forever 1"))
        .stdout(contains("infinite 1"))
        .stdout(contains("beauty 1"))
        .stdout(contains("sublime 1"));
}

#[test]
fn stdin_with_parallel_shorthand() {
    // Test the -p shorthand flag
    let assert = word_tally().write_stdin("hope forever").assert();
    assert
        .success()
        .stdout(contains("hope 1"))
        .stdout(contains("forever 1"));
}

#[test]
fn parallel_with_env_vars() {
    let assert = word_tally()
        .env("WORD_TALLY_CHUNK_SIZE", "4096")
        .env("WORD_TALLY_THREADS", "2")
        .write_stdin("hope infinite beauty forever sublime")
        .assert();

    assert
        .success()
        .stdout(contains("hope 1"))
        .stdout(contains("infinite 1"))
        .stdout(contains("beauty 1"))
        .stdout(contains("forever 1"))
        .stdout(contains("sublime 1"));
}

#[test]
fn parallel_with_large_chunk() {
    let assert = word_tally()
        .env("WORD_TALLY_CHUNK_SIZE", "65536")
        .write_stdin("truth beauty certain narrow sublime forever")
        .assert();

    assert
        .success()
        .stdout(contains("truth 1"))
        .stdout(contains("beauty 1"))
        .stdout(contains("certain 1"))
        .stdout(contains("narrow 1"))
        .stdout(contains("sublime 1"))
        .stdout(contains("forever 1"));
}

#[test]
#[cfg(unix)]
fn broken_pipe_behavior() {
    let input = r"Because I could not stop for Death –
He kindly stopped for me –
The Carriage held but just Ourselves –
And Immortality.";

    let mut cmd = word_tally();
    cmd.write_stdin(input)
        .assert()
        .success()
        .stdout(contains("Death"));
}

#[test]
fn normal_pipe_operation() {
    let input = r#""Hope" is the thing with feathers -
That perches in the soul -"#;

    let mut cmd = word_tally();
    cmd.write_stdin(input);
    cmd.assert()
        .success()
        .stdout(contains("Hope"))
        .stdout(contains("feathers"));
}

#[test]
#[cfg(unix)]
fn large_input_broken_pipe() {
    let poem = r"I dwell in Possibility – a fairer House than Prose –
More numerous of Windows – Superior – for Doors –

Of Chambers as the Cedars – Impregnable of eye –
And for an everlasting Roof
The Gambrels of the Sky –

Of Visitors – the fairest –
For Occupation – This –
The spreading wide my narrow Hands
To gather Paradise –

";
    let large_input = poem.repeat(100);
    word_tally().write_stdin(large_input).assert().success();
}

#[test]
fn single_file_input() {
    const CONTENT: &str = "narrow road narrow";
    let temp_file = NamedTempFile::new().expect("create temp file");
    fs::write(&temp_file, CONTENT).expect("write test file");

    word_tally()
        .arg(temp_file.path())
        .assert()
        .success()
        .stdout("narrow 2\nroad 1\n");
}

#[test]
fn multiple_file_inputs() {
    let temp_files = create_temp_files_with_content(&["narrow road", "road fame"]);

    let mut cmd = word_tally();
    for temp_file in &temp_files {
        cmd.arg(temp_file.path());
    }
    test_word_counts(cmd, &[("road", 2), ("narrow", 1), ("fame", 1)]);
}

#[test]
fn multi_file_with_mmap() {
    let temp_files = create_temp_files_with_content(&["narrow road", "road fame"]);

    let mut cmd = word_tally();
    cmd.arg("--io=parallel-mmap");
    for temp_file in &temp_files {
        cmd.arg(temp_file.path());
    }
    test_word_counts(cmd, &[("road", 2), ("narrow", 1), ("fame", 1)]);
}

#[test]
fn multi_file_with_parallel() {
    let temp_files = create_temp_files_with_content(&["narrow road", "road fame"]);

    let mut cmd = word_tally();
    for temp_file in &temp_files {
        cmd.arg(temp_file.path());
    }
    test_word_counts(cmd, &[("road", 2), ("narrow", 1), ("fame", 1)]);
}

#[test]
fn empty_args_defaults_to_stdin() {
    word_tally()
        .write_stdin("narrow fame narrow")
        .assert()
        .success()
        .stdout("narrow 2\nfame 1\n");
}

#[test]
fn stdin_with_file_as_sources() {
    // Create temporary file
    let temp_file = NamedTempFile::new().expect("create temp file");
    fs::write(&temp_file, "narrow road").expect("write test file");

    // Test mixing stdin (-) with a file path
    word_tally()
        .arg("-")
        .arg(temp_file.path())
        .write_stdin("road fame")
        .assert()
        .success()
        .stdout(contains("road 2\n"))
        .stdout(contains("narrow 1\n"))
        .stdout(contains("fame 1\n"));
}

#[test]
fn test_parallel_flags() {
    // Test default is parallel-stream
    word_tally()
        .write_stdin("test")
        .arg("--verbose")
        .assert()
        .success()
        .stderr(contains("io parallel-stream"));

    // Test explicit stream mode (sequential)
    word_tally()
        .write_stdin("test")
        .arg("--io=stream")
        .arg("--verbose")
        .assert()
        .success()
        .stderr(contains("io stream"));
}
