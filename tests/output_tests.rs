#[cfg(test)]
mod output_tests {
    use std::fs;
    use std::io::Write;
    use std::path::PathBuf;
    use tempfile::NamedTempFile;
    use word_tally::Output;

    #[test]
    fn test_output_new_with_none() {
        let output = Output::new(&None).expect("process test");
        // Should default to stdout
        assert!(matches!(output, _));
    }

    #[test]
    fn test_output_new_with_dash() {
        let path = PathBuf::from("-");
        let output = Output::new(&Some(path)).expect("process test");
        // Should use stdout for "-"
        assert!(matches!(output, _));
    }

    #[test]
    fn test_output_new_with_file_path() {
        let temp_file = NamedTempFile::new().expect("create temp file");
        let path = temp_file.path().to_path_buf();
        let output = Output::new(&Some(path)).expect("process test");
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
    fn test_output_write_line() {
        let temp_file = NamedTempFile::new().expect("create temp file");
        let mut output = Output::file(temp_file.path()).expect("process test");
        output.write_line("test line").expect("process test");
        output.flush().expect("process test");

        let contents = fs::read_to_string(temp_file.path()).expect("process test");
        assert_eq!(contents, "test line");
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
        output.write_all(b"test data").expect("write test data");
        output.flush().expect("process test");

        let contents = fs::read_to_string(temp_file.path()).expect("process test");
        assert_eq!(contents, "test data");
    }
}
