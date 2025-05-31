//! Tests for multi-file processing with different I/O strategies.

use assert_cmd::Command;
use predicates::str::contains;
use std::fs;
use tempfile::{NamedTempFile, tempdir};

fn word_tally() -> Command {
    Command::cargo_bin("word-tally").expect("run test")
}

fn create_test_files() -> (NamedTempFile, NamedTempFile) {
    let temp_file1 = NamedTempFile::new().expect("create temp file");
    let temp_file2 = NamedTempFile::new().expect("create temp file");
    fs::write(&temp_file1, "narrow road").expect("write test file");
    fs::write(&temp_file2, "road fame").expect("write test file");
    (temp_file1, temp_file2)
}

fn assert_common_output(cmd: &mut Command) {
    cmd.assert()
        .success()
        .stdout(contains("road 2"))
        .stdout(contains("narrow 1"))
        .stdout(contains("fame 1"));
}

#[test]
fn sequential_stream_multi_file() {
    let (temp_file1, temp_file2) = create_test_files();

    assert_common_output(
        word_tally()
            .arg("--io=stream")
            .arg(temp_file1.path())
            .arg(temp_file2.path()),
    );
}

#[test]
fn parallel_stream_multi_file() {
    let (temp_file1, temp_file2) = create_test_files();

    assert_common_output(
        word_tally()
            .arg("--io=parallel-stream")
            .arg(temp_file1.path())
            .arg(temp_file2.path()),
    );
}

#[test]
fn parallel_in_memory_multi_file() {
    let (temp_file1, temp_file2) = create_test_files();

    assert_common_output(
        word_tally()
            .arg("--io=parallel-in-memory")
            .arg(temp_file1.path())
            .arg(temp_file2.path()),
    );
}

#[test]
fn parallel_mmap_multi_file() {
    let (temp_file1, temp_file2) = create_test_files();

    assert_common_output(
        word_tally()
            .arg("--io=parallel-mmap")
            .arg(temp_file1.path())
            .arg(temp_file2.path()),
    );
}

#[test]
fn multi_file_with_stdin() {
    let temp_file = NamedTempFile::new().expect("create temp file");
    fs::write(&temp_file, "narrow road").expect("write test file");

    word_tally()
        .arg("--io=parallel-stream")
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
    let temp_dir = tempdir().expect("create temp dir");
    let temp_files: Vec<_> = (0..3)
        .map(|i| {
            let file_path = temp_dir.path().join(format!("file{i}.txt"));
            fs::write(&file_path, format!("word{i} common")).expect("write test file");
            file_path
        })
        .collect();

    let mut cmd = word_tally();
    cmd.arg("--io=parallel-mmap");

    for file_path in &temp_files {
        cmd.arg(file_path);
    }

    cmd.assert()
        .success()
        .stdout(contains("common 3"))
        .stdout(contains("word0 1"))
        .stdout(contains("word1 1"))
        .stdout(contains("word2 1"));
}
