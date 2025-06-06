//! Tests for View functionality.

use std::io::Write;

use tempfile::NamedTempFile;
use word_tally::{Metadata, View};

#[test]
fn test_view_from_mmap() {
    let test_data = b"Mmap test data";
    let mut temp_file = NamedTempFile::new().expect("create temp file");
    Write::write_all(&mut temp_file, test_data).expect("write test data");

    let view = View::try_from(temp_file.path()).expect("create test view");
    assert_eq!(view.path(), Some(temp_file.path()));
    assert_eq!(view.size(), Some(test_data.len() as u64));
    assert_eq!(view.as_ref(), test_data);
}

#[test]
fn test_view_from_bytes() {
    let test_data = b"Bytes test data";
    let view = View::from(&test_data[..]);
    assert_eq!(view.path(), None);
    assert_eq!(view.size(), Some(test_data.len() as u64));
    assert_eq!(view.as_ref(), test_data);
    assert_eq!(view.to_string(), "<bytes>");
}

#[test]
fn test_view_from_vec() {
    let vec_data: Vec<u8> = vec![1, 2, 3, 4, 5];
    let view = View::from(vec_data);
    assert_eq!(view.size(), Some(5));
    assert_eq!(view.path(), None);
}

#[test]
fn test_view_from_slice() {
    let slice_data: &[u8] = &[1, 2, 3, 4, 5];
    let view = View::from(slice_data);
    assert_eq!(view.size(), Some(5));
    assert_eq!(view.path(), None);
    assert_eq!(view.as_ref(), slice_data);
}

#[test]
fn test_view_from_string_bytes() {
    let string_data = String::from("Hello, world!");
    let view = View::from(string_data.as_bytes());
    assert_eq!(view.size(), Some(13));
}

#[test]
fn test_view_display() {
    let mut temp_file = NamedTempFile::new().expect("create temp file");
    Write::write_all(&mut temp_file, b"test").expect("write test data");

    let mmap_view = View::try_from(temp_file.path()).expect("create test view");
    let mmap_display = format!("{mmap_view}");
    assert_eq!(mmap_display, temp_file.path().display().to_string());

    let bytes_view = View::from(&b"test"[..]);
    assert_eq!(format!("{bytes_view}"), "<bytes>");
}

#[test]
fn test_view_mmap_nonexistent_file() {
    let result = View::try_from("/nonexistent/path/to/file.txt");
    assert!(result.is_err());
}

#[test]
fn test_view_file_not_found_error_message() {
    let nonexistent_path = "/nonexistent/path/to/file.txt";

    let view_result = View::try_from(nonexistent_path);
    assert!(view_result.is_err());
    let error = view_result.expect_err("should fail for nonexistent file");
    assert_eq!(
        error.to_string(),
        "I/O at /nonexistent/path/to/file.txt: no such file: /nonexistent/path/to/file.txt"
    );
}

#[test]
fn test_view_mmap_stdin_error() {
    let result = View::try_from("-");
    assert!(result.is_err());
    let error = result.expect_err("should fail for stdin mmap");
    assert_eq!(
        error.to_string(),
        "memory-mapped I/O requires a file, not stdin"
    );
}

#[test]
fn test_view_mmap_thread_safety() {
    let test_data = b"Thread safety test data for memory-mapped view";
    let mut temp_file = NamedTempFile::new().expect("create temp file");
    Write::write_all(&mut temp_file, test_data).expect("write test data");

    let view = View::try_from(temp_file.path()).expect("create test view");

    std::thread::scope(|s| {
        let handle1 = s.spawn(|| {
            if let View::Mmap { mmap, .. } = &view {
                let data = &**mmap;
                assert_eq!(data, test_data);
            }
        });

        let handle2 = s.spawn(|| {
            if let View::Mmap { mmap, .. } = &view {
                let data = &**mmap;
                assert_eq!(data, test_data);
            }
        });

        handle1.join().expect("thread 1 should complete");
        handle2.join().expect("thread 2 should complete");
    });
}

#[test]
fn test_view_as_ref() {
    let test_data = b"AsRef test data";

    // Test with bytes view
    let bytes_view = View::from(&test_data[..]);
    assert_eq!(bytes_view.as_ref(), test_data);

    // Test with mmap view
    let mut temp_file = NamedTempFile::new().expect("create temp file");
    Write::write_all(&mut temp_file, test_data).expect("write test data");

    let mmap_view = View::try_from(temp_file.path()).expect("create test view");
    assert_eq!(mmap_view.as_ref(), test_data);
}
