//! Tests for multi-file processing with different I/O strategies.

use assert_cmd::Command;
use predicates::str::contains;
use std::fs;
use tempfile::NamedTempFile;

fn word_tally() -> Command {
    Command::cargo_bin("word-tally").expect("run test")
}

#[test]
fn sequential_streamed_multi_file() {
    let temp_file1 = NamedTempFile::new().expect("create temp file");
    let temp_file2 = NamedTempFile::new().expect("create temp file");
    fs::write(&temp_file1, "narrow road").expect("write test file");
    fs::write(&temp_file2, "road fame").expect("write test file");

    word_tally()
        .arg("--io=streamed")
        .arg(temp_file1.path())
        .arg(temp_file2.path())
        .assert()
        .success()
        .stdout(contains("road 2"))
        .stdout(contains("narrow 1"))
        .stdout(contains("fame 1"));
}

#[test]
fn parallel_streamed_multi_file() {
    let temp_file1 = NamedTempFile::new().expect("create temp file");
    let temp_file2 = NamedTempFile::new().expect("create temp file");
    fs::write(&temp_file1, "narrow road").expect("write test file");
    fs::write(&temp_file2, "road fame").expect("write test file");

    word_tally()
        .arg("--io=streamed")
        .arg("--parallel")
        .arg(temp_file1.path())
        .arg(temp_file2.path())
        .assert()
        .success()
        .stdout(contains("road 2"))
        .stdout(contains("narrow 1"))
        .stdout(contains("fame 1"));
}

#[test]
fn parallel_buffered_multi_file() {
    let temp_file1 = NamedTempFile::new().expect("create temp file");
    let temp_file2 = NamedTempFile::new().expect("create temp file");
    fs::write(&temp_file1, "narrow road").expect("write test file");
    fs::write(&temp_file2, "road fame").expect("write test file");

    word_tally()
        .arg("--io=buffered")
        .arg("--parallel")
        .arg(temp_file1.path())
        .arg(temp_file2.path())
        .assert()
        .success()
        .stdout(contains("road 2"))
        .stdout(contains("narrow 1"))
        .stdout(contains("fame 1"));
}

#[test]
fn parallel_mmap_multi_file() {
    let temp_file1 = NamedTempFile::new().expect("create temp file");
    let temp_file2 = NamedTempFile::new().expect("create temp file");
    fs::write(&temp_file1, "narrow road").expect("write test file");
    fs::write(&temp_file2, "road fame").expect("write test file");

    word_tally()
        .arg("--io=mmap")
        .arg("--parallel")
        .arg(temp_file1.path())
        .arg(temp_file2.path())
        .assert()
        .success()
        .stdout(contains("road 2"))
        .stdout(contains("narrow 1"))
        .stdout(contains("fame 1"));
}

#[test]
fn multi_file_with_stdin() {
    let temp_file = NamedTempFile::new().expect("create temp file");
    fs::write(&temp_file, "narrow road").expect("write test file");

    word_tally()
        .arg("--io=streamed")
        .arg("-")
        .arg(temp_file.path())
        .write_stdin("road fame")
        .assert()
        .success()
        .stdout(contains("road 2"))
        .stdout(contains("narrow 1"))
        .stdout(contains("fame 1"));
}

#[test]
fn large_multi_file_sets() {
    let temp_files: Vec<NamedTempFile> = (0..5)
        .map(|i| {
            let temp_file = NamedTempFile::new().expect("create temp file");
            fs::write(&temp_file, format!("word{} common", i)).expect("write test file");
            temp_file
        })
        .collect();

    let mut cmd = word_tally();
    cmd.arg("--io=mmap").arg("--parallel");

    for temp_file in &temp_files {
        cmd.arg(temp_file.path());
    }

    cmd.assert()
        .success()
        .stdout(contains("common 5"))
        .stdout(contains("word0 1"))
        .stdout(contains("word1 1"))
        .stdout(contains("word2 1"))
        .stdout(contains("word3 1"))
        .stdout(contains("word4 1"));
}
