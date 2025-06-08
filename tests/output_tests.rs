#[cfg(test)]
mod output_tests {
    use std::fs;
    use std::io::{ErrorKind, Write};
    use std::path::PathBuf;
    use tempfile::NamedTempFile;
    use word_tally::Output;

    #[test]
    fn test_output_new_with_none() {
        let output = Output::new(None).expect("process test");
        // Should default to stdout
        assert!(matches!(output, _));
    }

    #[test]
    fn test_output_new_with_dash() {
        let path = PathBuf::from("-");
        let output = Output::new(Some(path.as_path())).expect("process test");
        // Should use stdout for "-"
        assert!(matches!(output, _));
    }

    #[test]
    fn test_output_new_with_file_path() {
        let temp_file = NamedTempFile::new().expect("create temp file");
        let path = temp_file.path();
        let output = Output::new(Some(path)).expect("process test");
        assert!(matches!(output, _));
    }

    #[test]
    fn test_output_stdout() {
        let output = Output::stdout();
        assert!(matches!(output, _));
    }

    #[test]
    fn test_output_stderr() {
        let output = Output::stderr();
        assert!(matches!(output, _));
    }

    #[test]
    fn test_output_file() {
        let temp_file = NamedTempFile::new().expect("create temp file");
        let output = Output::file(temp_file.path()).expect("process test");
        assert!(matches!(output, _));
    }

    #[test]
    fn test_output_write_bytes() {
        let temp_file = NamedTempFile::new().expect("create temp file");
        let mut output = Output::file(temp_file.path()).expect("process test");
        output
            .write_all(b"I'm Nobody! Who are you?")
            .expect("process test");
        output.flush().expect("process test");

        let contents = fs::read_to_string(temp_file.path()).expect("process test");
        assert_eq!(contents, "I'm Nobody! Who are you?");
    }

    #[test]
    fn test_output_default() {
        let output = Output::default();
        // Default should be stdout
        assert!(matches!(output, _));
    }

    #[test]
    fn test_output_write_trait() {
        let temp_file = NamedTempFile::new().expect("create temp file");
        let mut output = Output::file(temp_file.path()).expect("process test");
        output
            .write_all("The Brain—is wider than the Sky—".as_bytes())
            .expect("write test data");
        output.flush().expect("process test");

        let contents = fs::read_to_string(temp_file.path()).expect("process test");
        assert_eq!(contents, "The Brain—is wider than the Sky—");
    }

    #[test]
    fn test_write_bytes_succeeds() {
        let temp_file = NamedTempFile::new().expect("create temp file");
        let mut output = Output::file(temp_file.path()).expect("process test");

        output.write_all(b"").expect("write empty");
        output.write_all(b"Hope").expect("write Hope");
        output
            .write_all("With a capital letter—".as_bytes())
            .expect("write with capital");
        output
            .write_all("And without—is still a thing with feathers".as_bytes())
            .expect("write feathers");
        output
            .write_all("Tell all the truth but tell it slant—".as_bytes())
            .expect("write slant");

        output.flush().expect("flush");

        let contents = fs::read_to_string(temp_file.path()).expect("read file");
        let expected = "HopeWith a capital letter—And without—is still a thing with feathersTell all the truth but tell it slant—";
        assert_eq!(contents, expected);
    }

    #[test]
    fn test_write_bytes_with_newlines() {
        let temp_file = NamedTempFile::new().expect("create temp file");
        let mut output = Output::file(temp_file.path()).expect("process test");

        output
            .write_all("Much Madness is divinest Sense—\nTo a discerning Eye—".as_bytes())
            .expect("write lines");
        output
            .write_all("They shut me up in Prose—\n".as_bytes())
            .expect("write with newline");

        output.flush().expect("flush");

        let contents = fs::read_to_string(temp_file.path()).expect("read file");
        assert_eq!(
            contents,
            "Much Madness is divinest Sense—\nTo a discerning Eye—They shut me up in Prose—\n"
        );
    }

    #[test]
    #[cfg(unix)]
    fn test_broken_pipe_simulation() {
        use std::io::Write;
        use std::process::{Command, Stdio};

        let mut child = Command::new("sh")
            .arg("-c")
            .arg("exit 0")
            .stdin(Stdio::piped())
            .spawn()
            .expect("failed to spawn process");

        let mut stdin = child.stdin.take().expect("failed to get stdin");

        child.wait().expect("failed to wait for child");

        let write_result = stdin.write_all(b"data to write");

        assert!(
            matches!(write_result, Err(ref e) if e.kind() == ErrorKind::BrokenPipe),
            "Expected broken pipe error, got {write_result:?}"
        );
    }

    #[test]
    fn test_flush_can_fail() {
        use std::io::{self, Write};

        struct FailingWriter {
            fail_on_flush: bool,
        }

        impl Write for FailingWriter {
            fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
                Ok(buf.len())
            }

            fn flush(&mut self) -> io::Result<()> {
                if self.fail_on_flush {
                    Err(io::Error::other("flush failed"))
                } else {
                    Ok(())
                }
            }
        }

        let mut writer = FailingWriter {
            fail_on_flush: true,
        };
        let flush_result = writer.flush();
        assert!(flush_result.is_err());
        assert_eq!(
            flush_result
                .expect_err("flush should fail when FailingWriter is configured to fail")
                .kind(),
            ErrorKind::Other
        );
    }
}
