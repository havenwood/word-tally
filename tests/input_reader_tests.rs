//! Tests for input reader functionality.

use std::io::{BufRead, Read};
use tempfile::NamedTempFile;
use word_tally::{Input, Io};

#[test]
fn test_input_reader_mmap_basic() {
    let test_data = b"Hello, world! This is a test.";
    let mut temp_file = NamedTempFile::new().expect("create temp file");
    std::io::Write::write_all(&mut temp_file, test_data).expect("write test data");

    let input = Input::new(temp_file.path(), Io::ParallelMmap).expect("create test input");
    let mut reader = input.reader().expect("process test");

    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer).expect("process test");

    assert_eq!(buffer, test_data);
}

#[test]
fn test_input_reader_mmap_position_tracking() {
    let test_data = b"Hello, world!";
    let mut temp_file = NamedTempFile::new().expect("create temp file");
    std::io::Write::write_all(&mut temp_file, test_data).expect("write test data");

    let input = Input::new(temp_file.path(), Io::ParallelMmap).expect("create test input");
    let mut reader = input.reader().expect("process test");

    let mut buffer = [0u8; 5];
    let bytes_read = reader.read(&mut buffer).expect("process test");
    assert_eq!(bytes_read, 5);
    assert_eq!(&buffer, b"Hello");

    let mut buffer = [0u8; 7];
    let bytes_read = reader.read(&mut buffer).expect("process test");
    assert_eq!(bytes_read, 7);
    assert_eq!(&buffer, b", world");

    let mut buffer = [0u8; 10];
    let bytes_read = reader.read(&mut buffer).expect("process test");
    assert_eq!(bytes_read, 1);
    assert_eq!(buffer[0], b'!');

    let bytes_read = reader.read(&mut buffer).expect("process test");
    assert_eq!(bytes_read, 0);
}

#[test]
fn test_input_reader_mmap_empty_file() {
    let temp_file = NamedTempFile::new().expect("create temp file");

    let input = Input::new(temp_file.path(), Io::ParallelMmap).expect("create test input");
    let mut reader = input.reader().expect("process test");

    let mut buffer = [0u8; 10];
    let bytes_read = reader.read(&mut buffer).expect("process test");
    assert_eq!(bytes_read, 0);
}

#[test]
fn test_input_reader_bytes_basic() {
    let test_data = b"BytesReader test data";
    let input = Input::from_bytes(test_data);
    let mut reader = input.reader().expect("process test");

    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer).expect("process test");

    assert_eq!(buffer, test_data);
}

#[test]
fn test_input_reader_bytes_position_tracking() {
    let test_data = b"Bytes test";
    let input = Input::from_bytes(test_data);
    let mut reader = input.reader().expect("process test");

    let mut buffer = [0u8; 5];
    let bytes_read = reader.read(&mut buffer).expect("process test");
    assert_eq!(bytes_read, 5);
    assert_eq!(&buffer, b"Bytes");

    let mut buffer = [0u8; 5];
    let bytes_read = reader.read(&mut buffer).expect("process test");
    assert_eq!(bytes_read, 5);
    assert_eq!(&buffer, b" test");

    let bytes_read = reader.read(&mut buffer).expect("process test");
    assert_eq!(bytes_read, 0);
}

#[test]
fn test_input_reader_bytes_empty() {
    let input = Input::from_bytes([]);
    let mut reader = input.reader().expect("process test");

    let mut buffer = [0u8; 10];
    let bytes_read = reader.read(&mut buffer).expect("process test");
    assert_eq!(bytes_read, 0);
}

#[test]
fn test_input_reader_file() {
    let test_data = b"File test data";
    let mut temp_file = NamedTempFile::new().expect("create temp file");
    std::io::Write::write_all(&mut temp_file, test_data).expect("write test data");

    let input = Input::new(temp_file.path(), Io::ParallelStream).expect("create test input");
    let mut reader = input.reader().expect("process test");

    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer).expect("process test");
    assert_eq!(buffer, test_data);
}

#[test]
fn test_input_reader_stdin() {
    let input = Input::default();
    assert!(matches!(input, Input::Stdin));
    assert!(input.reader().is_ok());
}

#[test]
fn test_input_reader_bufread_impl() {
    let test_data = b"Line one\nLine two\nLine three";
    let input = Input::from_bytes(test_data);
    let reader = input.reader().expect("process test");

    let lines: Vec<String> = reader.lines().map(|r| r.expect("read line")).collect();
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0], "Line one");
    assert_eq!(lines[1], "Line two");
    assert_eq!(lines[2], "Line three");
}

#[test]
fn test_input_reader_bufread_fill_buf() {
    let test_data = b"Buffer test";
    let input = Input::from_bytes(test_data);
    let mut reader = input.reader().expect("process test");

    let buf = reader.fill_buf().expect("process test");
    assert!(!buf.is_empty());
    assert_eq!(&buf[0..6], b"Buffer");

    reader.consume(7);
    let buf = reader.fill_buf().expect("process test");
    assert_eq!(buf, b"test");

    reader.consume(4);
    let buf = reader.fill_buf().expect("process test");
    assert_eq!(buf.len(), 0);
}

#[test]
fn test_input_reader_zero_byte_read() {
    let test_data = b"test data";
    let mut temp_file = NamedTempFile::new().expect("create temp file");
    std::io::Write::write_all(&mut temp_file, test_data).expect("write test data");

    let input = Input::new(temp_file.path(), Io::ParallelMmap).expect("create test input");
    let mut reader = input.reader().expect("process test");

    let mut buffer = [0u8; 0];
    let bytes_read = reader.read(&mut buffer).expect("process test");
    assert_eq!(bytes_read, 0);

    let mut normal_buffer = [0u8; 4];
    let bytes_read = reader.read(&mut normal_buffer).expect("process test");
    assert_eq!(bytes_read, 4);
    assert_eq!(&normal_buffer, b"test");
}

#[test]
fn test_input_reader_multiple_eof_reads() {
    let test_data = b"EOF";
    let mut temp_file = NamedTempFile::new().expect("create temp file");
    std::io::Write::write_all(&mut temp_file, test_data).expect("write test data");

    let input = Input::new(temp_file.path(), Io::ParallelMmap).expect("create test input");
    let mut reader = input.reader().expect("process test");

    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer).expect("process test");
    assert_eq!(buffer, test_data);

    let mut buffer = [0u8; 10];
    let bytes_read = reader.read(&mut buffer).expect("process test");
    assert_eq!(bytes_read, 0);

    let bytes_read = reader.read(&mut buffer).expect("process test");
    assert_eq!(bytes_read, 0);
}

#[test]
fn test_input_reader_thread_safety() {
    let test_data = b"Thread-safe test";
    let mut temp_file = NamedTempFile::new().expect("create temp file");
    std::io::Write::write_all(&mut temp_file, test_data).expect("write test data");

    let input = Input::new(temp_file.path(), Io::ParallelMmap).expect("create test input");
    let thread_input = input.clone();

    std::thread::spawn(move || {
        let mut reader = thread_input.reader().expect("process test");
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer).expect("process test");
        assert_eq!(buffer, test_data);
    })
    .join()
    .expect("execute operation");

    let mut reader = input.reader().expect("process test");
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer).expect("process test");
    assert_eq!(buffer, test_data);
}

#[test]
fn test_mmap_reader_type() {
    let test_data = b"Test for MmapReader";
    let mut temp_file = NamedTempFile::new().expect("create temp file");
    std::io::Write::write_all(&mut temp_file, test_data).expect("write test data");

    let input = Input::new(temp_file.path(), Io::ParallelMmap).expect("create test input");
    let reader = input.reader().expect("process test");
    // Just verify we can create the reader successfully
    assert!(matches!(reader, word_tally::InputReader::Mmap(_)));
}

#[test]
fn test_bytes_reader_type() {
    let test_data = b"Test for BytesReader";
    let input = Input::from_bytes(test_data);
    let reader = input.reader().expect("process test");
    // Just verify we can create the reader successfully
    assert!(matches!(reader, word_tally::InputReader::Bytes(_)));
}

#[test]
fn test_bytes_reader_buffer_limit_8kb() {
    // Create a large buffer (16KB) to test the 8KB limit
    let large_data = vec![b'a'; 16 * 1024];
    let input = Input::from_bytes(large_data);

    let mut reader = input.reader().expect("process test");

    // fill_buf should only return 8KB at a time
    let buffer = reader.fill_buf().expect("process test");
    assert_eq!(buffer.len(), 8192, "Buffer should be limited to 8KB");

    // Consume half the buffer
    reader.consume(4096);

    // fill_buf should still return 8KB (4KB remaining from first + 4KB new)
    let buffer = reader.fill_buf().expect("process test");
    assert_eq!(
        buffer.len(),
        8192,
        "Buffer should still be 8KB after partial consume"
    );
}

#[test]
fn test_file_not_found_error_message() {
    let nonexistent_path = "/nonexistent/path/to/file.txt";
    let input =
        Input::new(nonexistent_path, word_tally::Io::ParallelStream).expect("create test input");

    let reader_result = input.reader();
    assert!(reader_result.is_err());

    let error = reader_result.expect_err("reader should fail for nonexistent file");
    assert_eq!(
        error.to_string(),
        "I/O at /nonexistent/path/to/file.txt: no such file: /nonexistent/path/to/file.txt"
    );
}

#[test]
#[cfg(unix)]
fn test_permission_denied_error_message() {
    use std::fs::{self, File};
    use std::os::unix::fs::PermissionsExt;
    use tempfile::TempDir;

    let temp_dir = TempDir::new().expect("process test");
    let file_path = temp_dir.path().join("test_no_permission.txt");

    // Create file with no read permissions
    {
        File::create(&file_path).expect("process test");
        let metadata = fs::metadata(&file_path).expect("process test");
        let mut perms = metadata.permissions();
        perms.set_mode(0o000); // No permissions
        fs::set_permissions(&file_path, perms).expect("process test");
    }

    let input = Input::new(&file_path, word_tally::Io::ParallelStream).expect("create test input");
    let reader_result = input.reader();
    assert!(reader_result.is_err());

    let error = reader_result.expect_err("reader should fail for nonexistent file");
    assert!(
        error.to_string().starts_with("I/O at ")
            && error.to_string().contains("permission denied: ")
    );

    // Clean up
    let metadata = fs::metadata(&file_path).expect("process test");
    let mut perms = metadata.permissions();
    perms.set_mode(0o644); // Restore permissions for cleanup
    fs::set_permissions(&file_path, perms).expect("process test");
}

#[test]
fn test_generic_io_error_message() {
    // This is harder to test since we need to trigger a non-specific I/O error
    // In practice, this would cover errors like disk full, etc.
    // We'll test that the error format is correct by testing with a mocked scenario
    // For now, we can at least verify the format works with our nonexistent file

    let nonexistent_path = "/dev/null/not_a_directory/file.txt";
    let input =
        Input::new(nonexistent_path, word_tally::Io::ParallelStream).expect("create test input");

    let reader_result = input.reader();
    if reader_result.is_err() {
        let error = reader_result.expect_err("reader should fail for nonexistent file");
        let error_string = error.to_string();
        // Should either be our specific error or fall back to generic format
        assert!(
            error_string.starts_with("I/O at ")
                && (error_string.contains("no such file: ")
                    || error_string.contains("permission denied: ")
                    || error_string.contains("failed to open file: "))
        );
    }
}

#[test]
fn test_slice_reader_consistency() {
    use memmap2::Mmap;
    use std::fs::File;
    use word_tally::input_reader::{BytesReader, MmapReader};

    let data = b"The fog comes\non little cat feet.\n\nIt sits looking\nover harbor and city\non silent haunches\nand then moves on.";

    // Create temporary file for mmap
    let mut temp_file = NamedTempFile::new().expect("create temp file");
    std::io::Write::write_all(&mut temp_file, data).expect("write test data");

    // Create mmap
    let file = File::open(temp_file.path()).expect("process test");
    #[allow(unsafe_code)]
    let mmap = unsafe { Mmap::map(&file).expect("process test") };

    // Test Read trait implementation consistency
    let mut mmap_reader = MmapReader::new(&mmap);
    let mut bytes_reader = BytesReader::new(data);

    let mut mmap_buf = vec![0; 20];
    let mut bytes_buf = vec![0; 20];

    // Read same amount from both
    let mmap_read = mmap_reader.read(&mut mmap_buf).expect("process test");
    let bytes_read = bytes_reader.read(&mut bytes_buf).expect("process test");

    assert_eq!(mmap_read, bytes_read);
    assert_eq!(mmap_buf, bytes_buf);

    // Test BufRead trait implementation consistency
    let mmap_reader = MmapReader::new(&mmap);
    let bytes_reader = BytesReader::new(data);

    // Read lines from both
    let mmap_lines: Vec<_> = mmap_reader
        .lines()
        .collect::<std::io::Result<_>>()
        .expect("process test");
    let bytes_lines: Vec<_> = bytes_reader
        .lines()
        .collect::<std::io::Result<_>>()
        .expect("execute operation");

    assert_eq!(mmap_lines, bytes_lines);

    // Test fill_buf consistency
    let mut mmap_reader = MmapReader::new(&mmap);
    let mut bytes_reader = BytesReader::new(data);

    let mmap_fill = mmap_reader.fill_buf().expect("process test");
    let bytes_fill = bytes_reader.fill_buf().expect("process test");

    assert_eq!(mmap_fill, bytes_fill);

    // Consume some data
    mmap_reader.consume(10);
    bytes_reader.consume(10);

    let mmap_fill = mmap_reader.fill_buf().expect("process test");
    let bytes_fill = bytes_reader.fill_buf().expect("process test");

    assert_eq!(mmap_fill, bytes_fill);
}

#[test]
fn test_slice_reader_large_buffer() {
    use memmap2::Mmap;
    use std::fs::File;
    use word_tally::input_reader::{BytesReader, MmapReader};

    // Define constants before any statements
    const BUFFER_SIZE: usize = 8192;

    let data = b"I heard a fly buzz when I died\n".repeat(1000);

    // Create temporary file for mmap
    let mut temp_file = NamedTempFile::new().expect("create temp file");
    std::io::Write::write_all(&mut temp_file, &data).expect("write test data");

    // Create mmap
    let file = File::open(temp_file.path()).expect("process test");
    #[allow(unsafe_code)]
    let mmap = unsafe { Mmap::map(&file).expect("process test") };

    let mut mmap_reader = MmapReader::new(&mmap);
    let mut bytes_reader = BytesReader::new(&data);

    let mmap_fill = mmap_reader.fill_buf().expect("process test");
    let bytes_fill = bytes_reader.fill_buf().expect("process test");

    assert_eq!(mmap_fill.len(), BUFFER_SIZE);
    assert_eq!(bytes_fill.len(), BUFFER_SIZE);
    assert_eq!(mmap_fill, bytes_fill);
}

#[test]
fn test_slice_reader_is_exhausted() {
    use word_tally::input_reader::BytesReader;

    let data = b"Brief is life but love is long";
    let mut reader = BytesReader::new(data.as_ref());

    // Initially not exhausted
    assert!(!reader.is_exhausted());

    // Read some data
    let mut buffer = vec![0u8; 10];
    let bytes_read = reader.read(&mut buffer).expect("process test");
    assert_eq!(bytes_read, 10);
    assert!(!reader.is_exhausted());

    // Read the rest
    let mut large_buffer = vec![0u8; 100];
    let bytes_read = reader.read(&mut large_buffer).expect("process test");
    assert_eq!(bytes_read, 20); // 30 total bytes - 10 already read
    assert!(reader.is_exhausted());

    // Additional reads should still be exhausted
    let bytes_read = reader.read(&mut buffer).expect("process test");
    assert_eq!(bytes_read, 0);
    assert!(reader.is_exhausted());
}
