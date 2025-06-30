//! Tests for metadata functionality.

use std::fs;

use word_tally::{Buffered, FileType, Mapped, Metadata};

#[test]
fn buffered_file_type() {
    let temp_file = tempfile::NamedTempFile::new().expect("create temp file");
    let buffered = Buffered::try_from(temp_file.path()).expect("create buffered");

    assert_eq!(buffered.file_type(), Some(FileType::File));
}

#[test]
fn buffered_stdin_file_type() {
    let stdin_buffered = Buffered::stdin();

    // On Unix, stdin might return a file type (e.g., CharacterDevice for terminal)
    // On non-Unix, it returns None
    #[cfg(unix)]
    {
        // The actual file type depends on how the test is run (terminal, pipe, etc.)
        // So we just check that it's either Some or None
        let _ = stdin_buffered.file_type();
    }
    #[cfg(not(unix))]
    {
        assert_eq!(stdin_buffered.file_type(), None);
    }
}

#[test]
fn mapped_file_type() {
    let temp_file = tempfile::NamedTempFile::new().expect("create temp file");
    fs::write(temp_file.path(), b"test data").expect("write test data");
    let mapped = Mapped::try_from(temp_file.path()).expect("create mapped");

    assert_eq!(mapped.file_type(), Some(FileType::File));
}

#[test]
fn mapped_memory_file_type() {
    let bytes = b"test data";
    let mapped = Mapped::from(bytes.as_slice());

    // Memory-mapped bytes have no file type
    assert_eq!(mapped.file_type(), None);
}

#[test]
fn mapped_directory_file_type() {
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let result = Mapped::try_from(temp_dir.path());

    // Should fail because directories can't be memory-mapped
    assert!(result.is_err());
    if let Err(e) = result {
        // The error should include the file type
        assert!(e.to_string().contains("directory"));
    }
}

#[cfg(unix)]
#[test]
fn buffered_directory_file_type() {
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let buffered = Buffered::try_from(temp_dir.path()).expect("create buffered");

    assert_eq!(buffered.file_type(), Some(FileType::Directory));
}

#[cfg(unix)]
#[test]
fn stdin_file_type_when_terminal() {
    use std::io::IsTerminal;

    let stdin_buffered = Buffered::stdin();
    let file_type = stdin_buffered.file_type();

    // If stdin is a terminal, it should be a character device
    if std::io::stdin().is_terminal() {
        assert_eq!(file_type, Some(FileType::CharacterDevice));
    }
    // Otherwise it could be a pipe, file, etc.
}
