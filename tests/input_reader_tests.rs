use std::io::{BufRead, Read};
use tempfile::NamedTempFile;
use word_tally::{Input, Io};

#[test]
fn test_input_reader_mmap_basic() {
    let test_data = b"Hello, world! This is a test.";
    let mut temp_file = NamedTempFile::new().unwrap();
    std::io::Write::write_all(&mut temp_file, test_data).unwrap();

    let input = Input::new(temp_file.path(), Io::MemoryMapped).unwrap();
    let mut reader = input.reader().unwrap();

    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer).unwrap();

    assert_eq!(buffer, test_data);
}

#[test]
fn test_input_reader_mmap_position_tracking() {
    let test_data = b"Hello, world!";
    let mut temp_file = NamedTempFile::new().unwrap();
    std::io::Write::write_all(&mut temp_file, test_data).unwrap();

    let input = Input::new(temp_file.path(), Io::MemoryMapped).unwrap();
    let mut reader = input.reader().unwrap();

    let mut buffer = [0u8; 5];
    let bytes_read = reader.read(&mut buffer).unwrap();
    assert_eq!(bytes_read, 5);
    assert_eq!(&buffer, b"Hello");

    let mut buffer = [0u8; 7];
    let bytes_read = reader.read(&mut buffer).unwrap();
    assert_eq!(bytes_read, 7);
    assert_eq!(&buffer, b", world");

    let mut buffer = [0u8; 10];
    let bytes_read = reader.read(&mut buffer).unwrap();
    assert_eq!(bytes_read, 1);
    assert_eq!(buffer[0], b'!');

    let bytes_read = reader.read(&mut buffer).unwrap();
    assert_eq!(bytes_read, 0);
}

#[test]
fn test_input_reader_mmap_empty_file() {
    let temp_file = NamedTempFile::new().unwrap();

    let input = Input::new(temp_file.path(), Io::MemoryMapped).unwrap();
    let mut reader = input.reader().unwrap();

    let mut buffer = [0u8; 10];
    let bytes_read = reader.read(&mut buffer).unwrap();
    assert_eq!(bytes_read, 0);
}

#[test]
fn test_input_reader_bytes_basic() {
    let test_data = b"BytesReader test data";
    let input = Input::from_bytes(test_data);
    let mut reader = input.reader().unwrap();

    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer).unwrap();

    assert_eq!(buffer, test_data);
}

#[test]
fn test_input_reader_bytes_position_tracking() {
    let test_data = b"Bytes test";
    let input = Input::from_bytes(test_data);
    let mut reader = input.reader().unwrap();

    let mut buffer = [0u8; 5];
    let bytes_read = reader.read(&mut buffer).unwrap();
    assert_eq!(bytes_read, 5);
    assert_eq!(&buffer, b"Bytes");

    let mut buffer = [0u8; 5];
    let bytes_read = reader.read(&mut buffer).unwrap();
    assert_eq!(bytes_read, 5);
    assert_eq!(&buffer, b" test");

    let bytes_read = reader.read(&mut buffer).unwrap();
    assert_eq!(bytes_read, 0);
}

#[test]
fn test_input_reader_bytes_empty() {
    let input = Input::from_bytes([]);
    let mut reader = input.reader().unwrap();

    let mut buffer = [0u8; 10];
    let bytes_read = reader.read(&mut buffer).unwrap();
    assert_eq!(bytes_read, 0);
}

#[test]
fn test_input_reader_file() {
    let test_data = b"File test data";
    let mut temp_file = NamedTempFile::new().unwrap();
    std::io::Write::write_all(&mut temp_file, test_data).unwrap();

    let input = Input::new(temp_file.path(), Io::Streamed).unwrap();
    let mut reader = input.reader().unwrap();

    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer).unwrap();
    assert_eq!(buffer, test_data);
}

#[test]
fn test_input_reader_stdin() {
    // Creating stdin input
    let input = Input::default();
    assert!(matches!(input, Input::Stdin));
    assert!(input.reader().is_ok());
}

#[test]
fn test_input_reader_bufread_impl() {
    let test_data = b"Line one\nLine two\nLine three";
    let input = Input::from_bytes(test_data);
    let reader = input.reader().unwrap();

    let lines: Vec<String> = reader.lines().map(|r| r.unwrap()).collect();
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0], "Line one");
    assert_eq!(lines[1], "Line two");
    assert_eq!(lines[2], "Line three");
}

#[test]
fn test_input_reader_bufread_fill_buf() {
    let test_data = b"Buffer test";
    let input = Input::from_bytes(test_data);
    let mut reader = input.reader().unwrap();

    let buf = reader.fill_buf().unwrap();
    assert!(!buf.is_empty());
    assert_eq!(&buf[0..6], b"Buffer");

    reader.consume(7);
    let buf = reader.fill_buf().unwrap();
    assert_eq!(buf, b"test");

    reader.consume(4);
    let buf = reader.fill_buf().unwrap();
    assert_eq!(buf.len(), 0);
}

#[test]
fn test_input_reader_zero_byte_read() {
    let test_data = b"test data";
    let mut temp_file = NamedTempFile::new().unwrap();
    std::io::Write::write_all(&mut temp_file, test_data).unwrap();

    let input = Input::new(temp_file.path(), Io::MemoryMapped).unwrap();
    let mut reader = input.reader().unwrap();

    let mut buffer = [0u8; 0];
    let bytes_read = reader.read(&mut buffer).unwrap();
    assert_eq!(bytes_read, 0);

    let mut normal_buffer = [0u8; 4];
    let bytes_read = reader.read(&mut normal_buffer).unwrap();
    assert_eq!(bytes_read, 4);
    assert_eq!(&normal_buffer, b"test");
}

#[test]
fn test_input_reader_multiple_eof_reads() {
    let test_data = b"EOF";
    let mut temp_file = NamedTempFile::new().unwrap();
    std::io::Write::write_all(&mut temp_file, test_data).unwrap();

    let input = Input::new(temp_file.path(), Io::MemoryMapped).unwrap();
    let mut reader = input.reader().unwrap();

    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer).unwrap();
    assert_eq!(buffer, test_data);

    let mut buffer = [0u8; 10];
    let bytes_read = reader.read(&mut buffer).unwrap();
    assert_eq!(bytes_read, 0);

    let bytes_read = reader.read(&mut buffer).unwrap();
    assert_eq!(bytes_read, 0);
}

#[test]
fn test_input_reader_thread_safety() {
    let test_data = b"Thread-safe test";
    let mut temp_file = NamedTempFile::new().unwrap();
    std::io::Write::write_all(&mut temp_file, test_data).unwrap();

    let input = Input::new(temp_file.path(), Io::MemoryMapped).unwrap();
    let thread_input = input.clone();

    std::thread::spawn(move || {
        let mut reader = thread_input.reader().unwrap();
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer).unwrap();
        assert_eq!(buffer, test_data);
    })
    .join()
    .unwrap();

    let mut reader = input.reader().unwrap();
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer).unwrap();
    assert_eq!(buffer, test_data);
}

#[test]
fn test_mmap_reader_type() {
    let test_data = b"Test for MmapReader";
    let mut temp_file = NamedTempFile::new().unwrap();
    std::io::Write::write_all(&mut temp_file, test_data).unwrap();

    let input = Input::new(temp_file.path(), Io::MemoryMapped).unwrap();
    let reader = input.reader().unwrap();
    // Just verify we can create the reader successfully
    assert!(matches!(reader, word_tally::InputReader::Mmap(_)));
}

#[test]
fn test_bytes_reader_type() {
    let test_data = b"Test for BytesReader";
    let input = Input::from_bytes(test_data);
    let reader = input.reader().unwrap();
    // Just verify we can create the reader successfully
    assert!(matches!(reader, word_tally::InputReader::Bytes(_)));
}

#[test]
fn test_bytes_reader_buffer_limit_8kb() {
    // Create a large buffer (16KB) to test the 8KB limit
    let large_data = vec![b'a'; 16 * 1024];
    let input = Input::from_bytes(large_data);

    let mut reader = input.reader().unwrap();

    // fill_buf should only return 8KB at a time
    let buffer = reader.fill_buf().unwrap();
    assert_eq!(buffer.len(), 8192, "Buffer should be limited to 8KB");

    // Consume half the buffer
    reader.consume(4096);

    // fill_buf should still return 8KB (4KB remaining from first + 4KB new)
    let buffer = reader.fill_buf().unwrap();
    assert_eq!(
        buffer.len(),
        8192,
        "Buffer should still be 8KB after partial consume"
    );
}

#[test]
fn test_file_not_found_error_message() {
    let nonexistent_path = "/nonexistent/path/to/file.txt";
    let input = Input::new(nonexistent_path, word_tally::Io::Streamed).unwrap();

    let reader_result = input.reader();
    assert!(reader_result.is_err());

    let error = reader_result.unwrap_err();
    assert_eq!(
        error.to_string(),
        "no such file: /nonexistent/path/to/file.txt"
    );
}

#[test]
#[cfg(unix)]
fn test_permission_denied_error_message() {
    use std::fs::{self, File};
    use std::os::unix::fs::PermissionsExt;
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test_no_permission.txt");

    // Create file with no read permissions
    {
        File::create(&file_path).unwrap();
        let metadata = fs::metadata(&file_path).unwrap();
        let mut perms = metadata.permissions();
        perms.set_mode(0o000); // No permissions
        fs::set_permissions(&file_path, perms).unwrap();
    }

    let input = Input::new(&file_path, word_tally::Io::Streamed).unwrap();
    let reader_result = input.reader();
    assert!(reader_result.is_err());

    let error = reader_result.unwrap_err();
    assert!(error.to_string().starts_with("permission denied: "));

    // Clean up
    let metadata = fs::metadata(&file_path).unwrap();
    let mut perms = metadata.permissions();
    perms.set_mode(0o644); // Restore permissions for cleanup
    fs::set_permissions(&file_path, perms).unwrap();
}

#[test]
fn test_generic_io_error_message() {
    // This is harder to test since we need to trigger a non-specific I/O error
    // In practice, this would cover errors like disk full, etc.
    // We'll test that the error format is correct by testing with a mocked scenario
    // For now, we can at least verify the format works with our nonexistent file

    let nonexistent_path = "/dev/null/not_a_directory/file.txt";
    let input = Input::new(nonexistent_path, word_tally::Io::Streamed).unwrap();

    let reader_result = input.reader();
    if reader_result.is_err() {
        let error = reader_result.unwrap_err();
        let error_string = error.to_string();
        // Should either be our specific error or fall back to generic format
        assert!(
            error_string.starts_with("no such file: ")
                || error_string.starts_with("permission denied: ")
                || error_string.starts_with("failed to open file: ")
        );
    }
}
