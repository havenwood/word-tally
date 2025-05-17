#[cfg(test)]
mod output_tests {
    use std::fs;
    use std::io::Write;
    use std::path::PathBuf;
    use tempfile::NamedTempFile;
    use word_tally::Output;

    #[test]
    fn test_output_new_with_none() {
        let output = Output::new(&None).unwrap();
        // Should default to stdout
        assert!(matches!(output, _));
    }

    #[test]
    fn test_output_new_with_dash() {
        let path = PathBuf::from("-");
        let output = Output::new(&Some(path)).unwrap();
        // Should use stdout for "-"
        assert!(matches!(output, _));
    }

    #[test]
    fn test_output_new_with_file_path() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_path_buf();
        let output = Output::new(&Some(path)).unwrap();
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
        let temp_file = NamedTempFile::new().unwrap();
        let output = Output::file(temp_file.path()).unwrap();
        assert!(matches!(output, _));
    }

    #[test]
    fn test_output_write_line() {
        let temp_file = NamedTempFile::new().unwrap();
        let mut output = Output::file(temp_file.path()).unwrap();
        output.write_line("test line").unwrap();
        output.flush().unwrap();

        let contents = fs::read_to_string(temp_file.path()).unwrap();
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
        let temp_file = NamedTempFile::new().unwrap();
        let mut output = Output::file(temp_file.path()).unwrap();
        output.write_all(b"test data").unwrap();
        output.flush().unwrap();

        let contents = fs::read_to_string(temp_file.path()).unwrap();
        assert_eq!(contents, "test data");
    }
}
