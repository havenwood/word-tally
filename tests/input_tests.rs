use std::io::Read;
use tempfile::NamedTempFile;
use word_tally::{Input, Io};

#[test]
fn test_input_new_stdin() {
    let input = Input::new("-", Io::Streamed).expect("create test input");
    assert!(matches!(input, Input::Stdin));
    assert_eq!(input.source(), "-");
    assert_eq!(input.size(), None);
}

#[test]
fn test_input_new_file() {
    let test_data = b"File test data";
    let mut temp_file = NamedTempFile::new().expect("create temp file");
    std::io::Write::write_all(&mut temp_file, test_data).expect("write test data");

    let input = Input::new(temp_file.path(), Io::Streamed).expect("create test input");
    assert!(matches!(input, Input::File(_)));

    let filename = temp_file
        .path()
        .file_name()
        .expect("process test")
        .to_str()
        .expect("process test");
    assert!(input.source().contains(filename));
    assert_eq!(input.size(), Some(test_data.len()));
}

#[test]
fn test_input_new_mmap() {
    let test_data = b"Memory mapped test data";
    let mut temp_file = NamedTempFile::new().expect("create temp file");
    std::io::Write::write_all(&mut temp_file, test_data).expect("write test data");

    let input = Input::new(temp_file.path(), Io::MemoryMapped).expect("create test input");
    assert!(matches!(input, Input::Mmap(_, _)));

    let filename = temp_file
        .path()
        .file_name()
        .expect("process test")
        .to_str()
        .expect("process test");
    assert!(input.source().contains(filename));
    assert_eq!(input.size(), Some(test_data.len()));
}

#[test]
fn test_input_from_bytes() {
    let test_data = b"Bytes test data";
    let input = Input::from_bytes(test_data);
    assert!(matches!(input, Input::Bytes(_)));
    assert_eq!(input.source(), "<bytes>");
    assert_eq!(input.size(), Some(test_data.len()));
}

#[test]
fn test_input_default() {
    let input = Input::default();
    assert!(matches!(input, Input::Stdin));
    assert_eq!(input.source(), "-");
    assert_eq!(input.size(), None);
}

#[test]
fn test_input_display() {
    let stdin_input = Input::default();
    assert_eq!(format!("{stdin_input}"), "Stdin");

    let mut temp_file = NamedTempFile::new().expect("create temp file");
    std::io::Write::write_all(&mut temp_file, b"test").expect("write test data");

    let file_input = Input::new(temp_file.path(), Io::Streamed).expect("create test input");
    let file_display = format!("{file_input}");
    assert!(file_display.starts_with("File("));
    assert!(file_display.contains(temp_file.path().display().to_string().as_str()));

    let mmap_input = Input::new(temp_file.path(), Io::MemoryMapped).expect("create test input");
    let mmap_display = format!("{mmap_input}");
    assert!(mmap_display.starts_with("Mmap("));
    assert!(mmap_display.contains(temp_file.path().display().to_string().as_str()));

    let bytes_input = Input::from_bytes(b"test");
    assert_eq!(format!("{bytes_input}"), "Bytes");
}

#[test]
fn test_input_clone() {
    let test_data = b"Clone test data";
    let mut temp_file = NamedTempFile::new().expect("create temp file");
    std::io::Write::write_all(&mut temp_file, test_data).expect("write test data");

    let input = Input::new(temp_file.path(), Io::MemoryMapped).expect("create test input");
    // Need the clone to test the Clone trait implementation works correctly
    #[allow(clippy::redundant_clone)]
    let cloned = input.clone();

    assert_eq!(input.source(), cloned.source());
    assert_eq!(input.size(), cloned.size());
    assert_eq!(format!("{input:?}"), format!("{:?}", cloned));
}

#[test]
fn test_input_bytes_error() {
    let result = Input::new("test.txt", Io::Bytes);
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("For byte data with `Io::Bytes`, use `Input::from_bytes()`")
    );
}

#[test]
fn test_input_reader_creation() {
    // Test that all input types can create readers successfully
    let stdin_input = Input::default();
    assert!(stdin_input.reader().is_ok());

    let test_data = b"test";
    let mut temp_file = NamedTempFile::new().expect("create temp file");
    std::io::Write::write_all(&mut temp_file, test_data).expect("write test data");

    let file_input = Input::new(temp_file.path(), Io::Streamed).expect("create test input");
    assert!(file_input.reader().is_ok());

    let mmap_input = Input::new(temp_file.path(), Io::MemoryMapped).expect("create test input");
    assert!(mmap_input.reader().is_ok());

    let bytes_input = Input::from_bytes(test_data);
    assert!(bytes_input.reader().is_ok());
}

#[test]
fn test_input_nonexistent_file() {
    let result = Input::new("/nonexistent/path/to/file.txt", Io::Streamed);
    assert!(result.is_ok()); // Input creation succeeds

    let input = result.expect("process test data");
    // But reader creation will fail
    let reader_result = input.reader();
    assert!(reader_result.is_err());
}

#[test]
fn test_input_mmap_nonexistent_file() {
    let result = Input::new("/nonexistent/path/to/file.txt", Io::MemoryMapped);
    assert!(result.is_err()); // Mmap fails immediately on file open
}

#[test]
fn test_input_buffered_io() {
    let test_data = b"Buffered test data";
    let mut temp_file = NamedTempFile::new().expect("create temp file");
    std::io::Write::write_all(&mut temp_file, test_data).expect("write test data");

    let input = Input::new(temp_file.path(), Io::Buffered).expect("create test input");
    assert!(matches!(input, Input::File(_)));

    let mut reader = input.reader().expect("process test");
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer).expect("process test");
    assert_eq!(buffer, test_data);
}

#[test]
fn test_input_source_edge_cases() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    // Test with path that has no filename
    let path = std::path::Path::new("/");
    let input = Input::new(path, Io::Streamed).expect("create test input");
    assert!(input.source().contains("No filename"));

    // Test with non-UTF8 filename (requires Unix-like system)
    #[cfg(unix)]
    {
        let non_utf8_bytes = b"\xFF\xFE\xFD";
        let non_utf8_osstr = OsStr::from_bytes(non_utf8_bytes);
        let path = std::path::Path::new(non_utf8_osstr);
        let input = Input::new(path, Io::Streamed).expect("create test input");
        assert!(input.source().contains("Non-UTF-8 filename"));
    }
}

#[test]
fn test_input_from_bytes_various_types() {
    // Test with Vec<u8>
    let vec_data: Vec<u8> = vec![1, 2, 3, 4, 5];
    let input = Input::from_bytes(vec_data);
    assert_eq!(input.size(), Some(5));

    // Test with &[u8]
    let slice_data: &[u8] = &[1, 2, 3, 4, 5];
    let input = Input::from_bytes(slice_data);
    assert_eq!(input.size(), Some(5));

    // Test with String
    let string_data = String::from("Hello, world!");
    let input = Input::from_bytes(string_data.as_bytes());
    assert_eq!(input.size(), Some(13));
}
