//! Tests for Reader functionality.

use std::io::Write;

use tempfile::NamedTempFile;
use word_tally::{Metadata, Reader};

#[test]
fn test_reader_from_file() {
    let test_data = b"File test data";
    let mut temp_file = NamedTempFile::new().expect("create temp file");
    Write::write_all(&mut temp_file, test_data).expect("write test data");

    let reader = Reader::try_from(temp_file.path()).expect("create test reader");
    assert_eq!(reader.path(), Some(temp_file.path()));
    assert_eq!(reader.size(), Some(test_data.len() as u64));
}

#[test]
fn test_reader_from_stdin() {
    let reader = Reader::stdin();
    assert_eq!(reader.path(), None);
    assert_eq!(reader.size(), None);
    assert_eq!(reader.to_string(), "-");
}

#[test]
fn test_reader_try_from_str_stdin() {
    let reader = Reader::try_from("-").expect("create stdin reader");
    assert_eq!(reader.path(), None);
    assert_eq!(reader.to_string(), "-");
}

#[test]
fn test_reader_try_from_pathbuf() {
    let mut temp_file = NamedTempFile::new().expect("create temp file");
    Write::write_all(&mut temp_file, b"test").expect("write test data");

    let pathbuf = temp_file.path().to_path_buf();
    let reader = Reader::try_from(pathbuf).expect("create reader");
    assert_eq!(reader.path(), Some(temp_file.path()));
}

#[test]
fn test_reader_with_buf_read() {
    let test_data = b"Line 1\nLine 2\nLine 3";
    let mut temp_file = NamedTempFile::new().expect("create temp file");
    Write::write_all(&mut temp_file, test_data).expect("write test data");

    let reader = Reader::try_from(temp_file.path()).expect("create reader");

    let mut lines = Vec::new();
    reader
        .with_buf_read(|buf_read| {
            for line in std::io::BufRead::lines(buf_read) {
                lines.push(line.expect("read line"));
            }
        })
        .expect("reader should not be poisoned");

    assert_eq!(lines, vec!["Line 1", "Line 2", "Line 3"]);
}

#[test]
fn test_reader_display() {
    let stdin_reader = Reader::stdin();
    assert_eq!(format!("{stdin_reader}"), "-");

    let mut temp_file = NamedTempFile::new().expect("create temp file");
    Write::write_all(&mut temp_file, b"test").expect("write test data");

    let file_reader = Reader::try_from(temp_file.path()).expect("create test reader");
    let file_display = format!("{file_reader}");
    assert_eq!(file_display, temp_file.path().display().to_string());
}

#[test]
fn test_reader_file_not_found() {
    let nonexistent_path = "/nonexistent/path/to/file.txt";

    let reader_result = Reader::try_from(nonexistent_path);
    assert!(reader_result.is_err());
    let error = reader_result.expect_err("should fail for nonexistent file");
    assert_eq!(
        error.to_string(),
        "no such file: /nonexistent/path/to/file.txt: /nonexistent/path/to/file.txt"
    );
}
