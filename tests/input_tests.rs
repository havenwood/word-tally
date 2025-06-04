//! Tests for input handling.

use std::io::Write;

use tempfile::NamedTempFile;
use word_tally::{Input, Io, Reader, View};

#[test]
fn test_input_new_stdin() {
    let input = Input::new("-", Io::ParallelStream).expect("create test input");
    assert!(matches!(input, Input::Reader(Reader::Stdin(_))));
    assert_eq!(input.source(), "-");
    assert_eq!(input.size(), None);
    assert_eq!(input.path(), None);
}

#[test]
fn test_input_new_file() {
    let test_data = b"File test data";
    let mut temp_file = NamedTempFile::new().expect("create temp file");
    Write::write_all(&mut temp_file, test_data).expect("write test data");

    let input = Input::new(temp_file.path(), Io::ParallelStream).expect("create test input");
    assert!(matches!(input, Input::Reader(Reader::File { .. })));

    let filename = temp_file
        .path()
        .file_name()
        .expect("process test")
        .to_str()
        .expect("process test");
    assert!(input.source().contains(filename));
    assert_eq!(input.size(), Some(test_data.len() as u64));
    assert_eq!(input.path(), Some(temp_file.path()));
}

#[test]
fn test_input_new_mmap() {
    let test_data = b"Memory mapped test data";
    let mut temp_file = NamedTempFile::new().expect("create temp file");
    Write::write_all(&mut temp_file, test_data).expect("write test data");

    let input = Input::new(temp_file.path(), Io::ParallelMmap).expect("create test input");
    assert!(matches!(input, Input::View(View::Mmap { .. })));

    let filename = temp_file
        .path()
        .file_name()
        .expect("process test")
        .to_str()
        .expect("process test");
    assert!(input.source().contains(filename));
    assert_eq!(input.size(), Some(test_data.len() as u64));
}

#[test]
fn test_input_from_bytes() {
    let test_data = b"Bytes test data";
    let input = Input::from(&test_data[..]);
    assert!(matches!(input, Input::View(View::Bytes(_))));
    assert_eq!(input.source(), "<bytes>");
    assert_eq!(input.size(), Some(test_data.len() as u64));
}

#[test]
fn test_input_default() {
    let input = Input::default();
    assert!(matches!(input, Input::Reader(Reader::Stdin(_))));
    assert_eq!(input.source(), "-");
    assert_eq!(input.size(), None);
}

#[test]
fn test_input_display() {
    let stdin_input = Input::default();
    assert_eq!(format!("{stdin_input}"), "Stdin");

    let mut temp_file = NamedTempFile::new().expect("create temp file");
    Write::write_all(&mut temp_file, b"test").expect("write test data");

    let file_input = Input::new(temp_file.path(), Io::ParallelStream).expect("create test input");
    let file_display = format!("{file_input}");
    assert!(file_display.starts_with("File("));
    assert!(file_display.contains(temp_file.path().display().to_string().as_str()));

    let mmap_input = Input::new(temp_file.path(), Io::ParallelMmap).expect("create test input");
    let mmap_display = format!("{mmap_input}");
    assert!(mmap_display.starts_with("Mmap("));
    assert!(mmap_display.contains(temp_file.path().display().to_string().as_str()));

    let bytes_input = Input::from(&b"test"[..]);
    assert_eq!(format!("{bytes_input}"), "Bytes");
}

#[test]
fn test_input_bytes_error() {
    let result = Input::new("test.txt", Io::ParallelBytes);
    assert!(result.is_err());
    assert!(
        result
            .expect_err("Input::new with ParallelBytes should fail")
            .to_string()
            .contains("byte I/O mode requires `Input::from()`")
    );
}

#[test]
fn test_input_nonexistent_file() {
    let result = Input::new("/nonexistent/path/to/file.txt", Io::ParallelStream);
    assert!(result.is_err()); // File opening now happens in Input::new
}

#[test]
fn test_input_mmap_nonexistent_file() {
    let result = Input::new("/nonexistent/path/to/file.txt", Io::ParallelMmap);
    assert!(result.is_err()); // Mmap fails immediately on file open
}

#[test]
fn test_input_file_not_found_error_message() {
    let nonexistent_path = "/nonexistent/path/to/file.txt";

    // Test both stream and mmap modes
    for io_mode in [Io::ParallelStream, Io::ParallelMmap] {
        let result = Input::new(nonexistent_path, io_mode);
        assert!(result.is_err());
        let error = result.expect_err("should fail for nonexistent file");
        assert_eq!(
            error.to_string(),
            "I/O at /nonexistent/path/to/file.txt: no such file: /nonexistent/path/to/file.txt"
        );
    }
}

#[test]
#[cfg(unix)]
fn test_input_permission_denied_error_message() {
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
        perms.set_mode(0o000);
        fs::set_permissions(&file_path, perms).expect("process test");
    }

    // Test both stream and mmap modes
    for io_mode in [Io::ParallelStream, Io::ParallelMmap] {
        let result = Input::new(&file_path, io_mode);
        assert!(result.is_err());
        let error = result.expect_err("should fail for permission denied");
        assert!(
            error.to_string().starts_with("I/O at ")
                && error.to_string().contains("permission denied: ")
        );
    }

    // Clean up
    let metadata = fs::metadata(&file_path).expect("process test");
    let mut perms = metadata.permissions();
    perms.set_mode(0o644);
    fs::set_permissions(&file_path, perms).expect("process test");
}

#[test]
fn test_input_in_memory_io() {
    let test_data = b"In-memory test data";
    let mut temp_file = NamedTempFile::new().expect("create temp file");
    Write::write_all(&mut temp_file, test_data).expect("write test data");

    let input = Input::new(temp_file.path(), Io::ParallelInMemory).expect("create test input");
    assert!(matches!(input, Input::Reader(Reader::File { .. })));
}

#[test]
fn test_input_source_edge_cases() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    // Test with root path
    let path = std::path::Path::new("/");
    let input = Input::new(path, Io::ParallelStream).expect("create test input");
    assert_eq!(input.source(), "/");

    // Test with non-UTF8 path (requires Unix-like system)
    #[cfg(unix)]
    {
        let non_utf8_bytes = b"\xFF\xFE\xFD";
        let non_utf8_osstr = OsStr::from_bytes(non_utf8_bytes);
        let path = std::path::Path::new(non_utf8_osstr);
        // The file doesn't exist, so Input::new should fail
        let result = Input::new(path, Io::ParallelStream);
        assert!(result.is_err());
        // Check the error message contains the lossy path
        if let Err(e) = result {
            assert!(e.to_string().contains("���"));
        }
    }
}

#[test]
fn test_input_from_bytes_various_types() {
    // Test with Vec<u8>
    let vec_data: Vec<u8> = vec![1, 2, 3, 4, 5];
    let input = Input::from(vec_data);
    assert_eq!(input.size(), Some(5));

    // Test with &[u8]
    let slice_data: &[u8] = &[1, 2, 3, 4, 5];
    let input = Input::from(slice_data);
    assert_eq!(input.size(), Some(5));
    assert_eq!(input.path(), None);

    // Test AsRef<[u8]> for View
    if let Input::View(view) = &input {
        assert_eq!(view.as_ref(), slice_data);
    } else {
        unreachable!("expected view variant");
    }

    // Test with String
    let string_data = String::from("Hello, world!");
    let input = Input::from(string_data.as_bytes());
    assert_eq!(input.size(), Some(13));
}

#[test]
fn test_mmap_thread_safety() {
    use std::thread;

    let test_data = b"Thread safe memory mapped data";
    let mut temp_file = NamedTempFile::new().expect("create temp file");
    Write::write_all(&mut temp_file, test_data).expect("write test data");

    let input = Input::new(temp_file.path(), Io::ParallelMmap).expect("create test input");

    // Verify we can read from MmapView in multiple threads using scoped threads
    thread::scope(|s| {
        let handle1 = s.spawn(|| {
            // Each thread accesses the data through the input's as_bytes() method
            let Input::View(View::Mmap { mmap, .. }) = &input else {
                unreachable!("expected mmap view variant")
            };
            let data = &**mmap;
            assert_eq!(data.len(), test_data.len());
            assert_eq!(&data[0..6], b"Thread");
        });

        let handle2 = s.spawn(|| {
            let Input::View(View::Mmap { mmap, .. }) = &input else {
                unreachable!("expected mmap view variant")
            };
            let data = &**mmap;
            assert_eq!(data.len(), test_data.len());
            assert_eq!(&data[7..11], b"safe");
        });

        handle1.join().expect("Thread 1 failed");
        handle2.join().expect("Thread 2 failed");
    });
}

#[test]
fn test_view_helper_methods() {
    let test_data = b"Test data for view";
    let mut temp_file = NamedTempFile::new().expect("create temp file");
    Write::write_all(&mut temp_file, test_data).expect("write test data");

    // Test Mmap view
    let input = Input::new(temp_file.path(), Io::ParallelMmap).expect("create test input");
    if let Input::View(view) = input {
        assert_eq!(view.path(), Some(temp_file.path()));
        assert_eq!(view.size(), test_data.len() as u64);
        assert_eq!(view.as_ref(), test_data);
    } else {
        unreachable!("expected view variant");
    }

    // Test Bytes view
    let bytes_input = Input::from(test_data.as_slice());
    if let Input::View(view) = bytes_input {
        assert_eq!(view.path(), None);
        assert_eq!(view.size(), test_data.len() as u64);
        assert_eq!(view.as_ref(), test_data);
    } else {
        unreachable!("expected view variant");
    }
}

#[test]
fn test_reader_helper_methods() {
    let test_data = b"Test data for reader";
    let mut temp_file = NamedTempFile::new().expect("create temp file");
    Write::write_all(&mut temp_file, test_data).expect("write test data");

    // Test File reader
    let input = Input::new(temp_file.path(), Io::Stream).expect("create test input");
    if let Input::Reader(reader) = &input {
        assert_eq!(reader.path(), Some(temp_file.path()));

        // Test with_buf_read
        reader.with_buf_read(|buf_read| {
            let mut content = String::new();
            std::io::Read::read_to_string(buf_read, &mut content).expect("read content");
            assert_eq!(content.as_bytes(), test_data);
        });
    } else {
        unreachable!("expected reader variant");
    }

    // Test Stdin reader
    let stdin_input = Input::new("-", Io::Stream).expect("create test input");
    if let Input::Reader(reader) = stdin_input {
        assert_eq!(reader.path(), None);
    } else {
        unreachable!("expected reader variant");
    }
}

#[test]
fn test_reader_with_buf_read_sequential_access() {
    use std::io::BufRead;

    let temp_file = NamedTempFile::new().expect("create temp file");
    std::fs::write(temp_file.path(), "line1\nline2\nline3").expect("write test data");

    let input = Input::new(temp_file.path(), Io::ParallelStream).expect("create input");

    if let Input::Reader(reader) = &input {
        let mut lines = Vec::new();
        reader.with_buf_read(|buf_read| {
            for line in buf_read.lines() {
                lines.push(line.expect("read line"));
            }
        });
        assert_eq!(lines, vec!["line1", "line2", "line3"]);
    }
}

#[test]
fn test_reader_with_buf_read_multiple_calls() {
    let temp_file = NamedTempFile::new().expect("create temp file");
    std::fs::write(temp_file.path(), "hello world").expect("write test data");

    let input = Input::new(temp_file.path(), Io::ParallelStream).expect("create input");

    if let Input::Reader(reader) = &input {
        let first_read = reader.with_buf_read(|buf_read| {
            let mut buffer = [0u8; 5];
            buf_read.read_exact(&mut buffer).expect("read bytes");
            String::from_utf8(buffer.to_vec()).expect("valid UTF-8")
        });
        assert_eq!(first_read, "hello");

        let second_read = reader.with_buf_read(|buf_read| {
            let mut buffer = String::new();
            buf_read
                .read_to_string(&mut buffer)
                .expect("read remaining");
            buffer
        });
        assert_eq!(second_read, " world");
    }
}

#[test]
fn test_reader_with_buf_read_error_propagation() {
    use std::io::{Error, ErrorKind};

    let temp_file = NamedTempFile::new().expect("create temp file");
    std::fs::write(temp_file.path(), "test data").expect("write test data");

    let input = Input::new(temp_file.path(), Io::ParallelStream).expect("create input");

    if let Input::Reader(reader) = &input {
        let result = reader
            .with_buf_read(|_buf_read| -> Result<(), Error> { Err(Error::other("test error")) });

        assert!(result.is_err());
        assert_eq!(
            result.expect_err("should be error").kind(),
            ErrorKind::Other
        );
    }
}

#[test]
fn test_stdin_reader_with_buf_read() {
    let stdin_input = Input::default();

    if let Input::Reader(reader) = &stdin_input {
        let result = reader.with_buf_read(|_buf_read| 42);
        assert_eq!(result, 42);
    }
}
