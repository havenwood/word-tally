//! Tests for Mapped functionality.

use std::io::Write;

use tempfile::NamedTempFile;
use word_tally::{Mapped, Metadata};

#[test]
fn view_from_mmap() {
    let test_data = b"Mmap test data";
    let mut temp_file = NamedTempFile::new().expect("create temp file");
    Write::write_all(&mut temp_file, test_data).expect("write test data");

    let view = Mapped::try_from(temp_file.path()).expect("create test view");
    assert_eq!(view.path(), Some(temp_file.path()));
    assert_eq!(view.len(), test_data.len());
    assert_eq!(&*view, test_data);
}

#[test]
fn view_from_bytes() {
    let test_data = b"Bytes test data";
    let view = Mapped::from(&test_data[..]);
    assert_eq!(view.path(), None);
    assert_eq!(view.len(), test_data.len());
    assert_eq!(&*view, test_data);
    assert_eq!(view.to_string(), "<bytes>");
}

#[test]
fn view_from_vec() {
    let vec_data: Vec<u8> = vec![1, 2, 3, 4, 5];
    let view = Mapped::from(vec_data);
    assert_eq!(view.len(), 5);
    assert_eq!(view.path(), None);
}

#[test]
fn view_from_slice() {
    let slice_data: &[u8] = &[1, 2, 3, 4, 5];
    let view = Mapped::from(slice_data);
    assert_eq!(view.len(), 5);
    assert_eq!(view.path(), None);
    assert_eq!(&*view, slice_data);
}

#[test]
fn view_from_string_bytes() {
    let string_data = String::from("Hello, world!");
    let view = Mapped::from(string_data.as_bytes());
    assert_eq!(view.len(), 13);
}

#[test]
fn view_display() {
    let mut temp_file = NamedTempFile::new().expect("create temp file");
    Write::write_all(&mut temp_file, b"test").expect("write test data");

    let mmap_view = Mapped::try_from(temp_file.path()).expect("create test view");
    let mmap_display = format!("{mmap_view}");
    assert_eq!(mmap_display, temp_file.path().display().to_string());

    let bytes_view = Mapped::from(&b"test"[..]);
    assert_eq!(format!("{bytes_view}"), "<bytes>");
}

#[test]
fn view_mmap_nonexistent_file() {
    // Use a path that's guaranteed not to exist on any platform
    let temp_dir = std::env::temp_dir();
    let nonexistent_path = temp_dir.join("nonexistent_word_tally_test_file_12345.txt");

    // Ensure the file doesn't exist
    if nonexistent_path.exists() {
        std::fs::remove_file(&nonexistent_path).ok();
    }

    let result = Mapped::try_from(nonexistent_path.as_path());
    assert!(result.is_err());
}

#[test]
fn view_file_not_found_error_message() {
    // Use a path that's guaranteed not to exist on any platform
    let temp_dir = std::env::temp_dir();
    let nonexistent_path = temp_dir.join("nonexistent_word_tally_test_file_67890.txt");

    // Ensure the file doesn't exist
    if nonexistent_path.exists() {
        std::fs::remove_file(&nonexistent_path).ok();
    }

    let nonexistent_path_str = nonexistent_path
        .to_str()
        .expect("path should be valid utf-8");
    let view_result = Mapped::try_from(nonexistent_path.as_path());
    assert!(view_result.is_err());
    let error = view_result.expect_err("should fail for nonexistent file");
    assert_eq!(
        error.to_string(),
        format!("no such file: {nonexistent_path_str}")
    );
}

#[test]
fn view_mmap_dash_error() {
    // "-" represents stdin, which cannot be memory-mapped
    let result = Mapped::try_from("-");
    assert!(result.is_err());
    let error = result.expect_err("should fail for stdin");
    assert_eq!(error.to_string(), "cannot memory map stdin");
}

#[test]
fn view_mmap_thread_safety() {
    let test_data = b"Thread safety test data for memory-mapped view";
    let mut temp_file = NamedTempFile::new().expect("create temp file");
    Write::write_all(&mut temp_file, test_data).expect("write test data");

    let view = Mapped::try_from(temp_file.path()).expect("create test view");

    std::thread::scope(|s| {
        let handle1 = s.spawn(|| {
            assert!(
                matches!(&view, Mapped::File { .. }),
                "expected File variant for mmap test"
            );
            if let Mapped::File { mmap, .. } = &view {
                let data = &**mmap;
                assert_eq!(data, test_data);
            }
        });

        let handle2 = s.spawn(|| {
            assert!(
                matches!(&view, Mapped::File { .. }),
                "expected File variant for mmap test"
            );
            if let Mapped::File { mmap, .. } = &view {
                let data = &**mmap;
                assert_eq!(data, test_data);
            }
        });

        handle1.join().expect("thread 1 should complete");
        handle2.join().expect("thread 2 should complete");
    });
}

#[test]
fn view_as_ref() {
    let test_data = b"AsRef test data";

    // Test with bytes view
    let bytes_view = Mapped::from(&test_data[..]);
    assert_eq!(&*bytes_view, test_data);

    // Test with mmap view
    let mut temp_file = NamedTempFile::new().expect("create temp file");
    Write::write_all(&mut temp_file, test_data).expect("write test data");

    let mmap_view = Mapped::try_from(temp_file.path()).expect("create test view");
    assert_eq!(&*mmap_view, test_data);
}

#[test]
fn not_mappable_error_directory() {
    let temp_dir = std::env::temp_dir();
    let result = Mapped::try_from(temp_dir.as_path());
    assert!(result.is_err());

    let error = result.expect_err("should fail for directory");
    let error_msg = error.to_string();
    assert!(error_msg.contains("cannot memory map directory"));
    assert!(error_msg.contains(temp_dir.to_str().expect("path should be valid utf-8")));
}

#[cfg(unix)]
#[test]
fn not_mappable_error_character_device() {
    // This test uses Unix-specific /dev/null to test character device error handling
    let result = Mapped::try_from("/dev/null");
    assert!(result.is_err());

    let error = result.expect_err("should fail for character device");
    assert_eq!(
        error.to_string(),
        "cannot memory map character device: /dev/null"
    );
}

#[cfg(unix)]
#[test]
fn not_mappable_error_fifo() {
    use std::process::Command;

    // Create a named pipe in temp directory
    let temp_dir = std::env::temp_dir();
    let fifo_path = temp_dir.join("test_fifo");
    let fifo_path_str = fifo_path.to_str().expect("path should be valid utf-8");

    // Remove any existing FIFO
    drop(std::fs::remove_file(&fifo_path));

    // Create FIFO
    let output = Command::new("mkfifo")
        .arg(&fifo_path)
        .output()
        .expect("failed to execute mkfifo");

    assert!(output.status.success(), "mkfifo command should succeed");

    // Test error message
    let result = Mapped::try_from(fifo_path.as_path());
    assert!(result.is_err());

    let error = result.expect_err("should fail for FIFO");
    assert_eq!(
        error.to_string(),
        format!("cannot memory map named pipe: {fifo_path_str}")
    );

    // Cleanup
    drop(std::fs::remove_file(&fifo_path));
}
