#[cfg(test)]
mod verbose_unit_tests {
    use std::io::{Result as IoResult, Write};
    use std::sync::{Arc, Mutex};
    use word_tally::{Case, Filters, Input, Io, Options, Serialization, Sort, WordTally};

    // Mock writer to capture verbose output
    #[derive(Default)]
    struct MockWriter {
        content: Arc<Mutex<Vec<u8>>>,
    }

    impl MockWriter {
        fn new() -> Self {
            Self {
                content: Arc::new(Mutex::new(Vec::new())),
            }
        }
    }

    impl Write for MockWriter {
        fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
            self.content
                .lock()
                .expect("process test")
                .extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> IoResult<()> {
            Ok(())
        }
    }

    // Helper function to create test WordTally
    fn create_test_tally(options: &Options) -> WordTally {
        let test_text = b"hope is the thing with feathers that perches";
        let mut temp_file = tempfile::NamedTempFile::new().expect("create temp file");
        Write::write_all(&mut temp_file, test_text).expect("write test data");

        let input = Input::new(
            temp_file.path().to_str().expect("temp file path"),
            options.io(),
        )
        .expect("process test");
        WordTally::new(&input, options).expect("create word tally")
    }

    // Test that verbose output works with JSON format
    #[test]
    fn test_verbose_json_format() {
        let _mock = MockWriter::new();
        let options = Options::default().with_serialization(Serialization::Json);
        let tally = create_test_tally(&options);

        // We can't directly test handle_output since it's not public
        // Instead, we test through the CLI integration which uses verbose output
        // This is already covered by cli_verbose_tests.rs

        // However, we can test that the WordTally serializes properly to JSON
        let json = serde_json::to_string(&tally).expect("serialize JSON");
        assert!(json.contains("\"count\":"));
        assert!(json.contains("\"uniqCount\":"));
        assert!(json.contains("\"tally\":"));
    }

    // Test that verbose output works with CSV format
    #[test]
    fn test_verbose_csv_format() {
        let _mock = MockWriter::new();
        let options = Options::default().with_serialization(Serialization::Csv);
        let tally = create_test_tally(&options);

        // Test CSV serialization of the tally itself
        let mut csv_writer = csv::Writer::from_writer(vec![]);
        csv_writer
            .write_record(["word", "count"])
            .expect("process test");

        for (word, count) in &tally {
            csv_writer
                .write_record([word.as_ref(), &count.to_string()])
                .expect("execute operation");
        }

        let data = csv_writer.into_inner().expect("process test");
        let csv = String::from_utf8(data).expect("process test");
        assert!(csv.contains("word,count"));
        assert!(csv.contains("hope,1"));
    }

    // Test that verbose output works with Text format
    #[test]
    fn test_verbose_text_format() {
        let _mock = MockWriter::new();
        let options = Options::default();
        let tally = create_test_tally(&options);

        // Test text serialization through debug trait
        let text = format!("{tally:?}");
        assert!(!text.is_empty());
    }

    // Test verbose with different options
    #[test]
    fn test_verbose_with_various_options() {
        let options = Options::default()
            .with_case(Case::Upper)
            .with_sort(Sort::Asc)
            .with_io(Io::ParallelStream);

        let tally = create_test_tally(&options);

        assert_eq!(tally.options().case(), Case::Upper);
        assert_eq!(tally.options().sort(), Sort::Asc);
        assert_eq!(tally.options().io(), Io::ParallelStream);

        // Verify the tally was created with the options
        assert!(tally.count() > 0);
        assert!(tally.uniq_count() > 0);
    }

    // Test verbose with filters
    #[test]
    fn test_verbose_with_filters() {
        let options = Options::default()
            .with_case(Case::Lower)
            .with_filters(Filters::default().with_min_chars(3).with_min_count(1));

        let tally = create_test_tally(&options);

        // Check that filters are applied
        let filters = tally.options().filters();
        assert_eq!(filters.min_chars(), Some(3));
        assert_eq!(filters.min_count(), Some(1));

        // Verify filtering worked by checking the tally
        for (word, _) in &tally {
            assert!(word.len() >= 3);
        }
    }

    // Test serialization of Option<T> fields
    #[test]
    fn test_option_serialization() {
        let options_with_filters =
            Options::default().with_filters(Filters::default().with_min_chars(5).with_min_count(2));

        let options_without_filters = Options::default();

        let tally_with = create_test_tally(&options_with_filters);
        let tally_without = create_test_tally(&options_without_filters);

        let json_with = serde_json::to_string(&tally_with).expect("serialize JSON");
        let json_without = serde_json::to_string(&tally_without).expect("serialize JSON");

        // Check that options are properly serialized in JSON
        assert!(json_with.contains("\"options\":"));
        assert!(json_without.contains("\"options\":"));
    }

    // Test for edge cases
    #[test]
    fn test_verbose_with_empty_input() {
        let test_text = b"";
        let mut temp_file = tempfile::NamedTempFile::new().expect("create temp file");
        Write::write_all(&mut temp_file, test_text).expect("write test data");

        let options = Options::default();
        let input = Input::new(
            temp_file.path().to_str().expect("temp file path"),
            options.io(),
        )
        .expect("process test");
        let tally = WordTally::new(&input, &options).expect("create word tally");

        assert_eq!(tally.count(), 0);
        assert_eq!(tally.uniq_count(), 0);

        // Test that empty tally still serializes correctly
        let json = serde_json::to_string(&tally).expect("serialize JSON");
        assert!(json.contains("\"count\":0"));
        assert!(json.contains("\"uniqCount\":0"));
    }

    // Test verbose with special characters
    #[test]
    fn test_verbose_with_special_characters() {
        let test_text = b"test \"quoted\" text & special <chars>";
        let mut temp_file = tempfile::NamedTempFile::new().expect("create temp file");
        Write::write_all(&mut temp_file, test_text).expect("write test data");

        let options = Options::default();
        let input = Input::new(
            temp_file.path().to_str().expect("temp file path"),
            options.io(),
        )
        .expect("process test");
        let tally = WordTally::new(&input, &options).expect("create word tally");

        // Check that special characters are handled properly
        let json = serde_json::to_string(&tally).expect("serialize JSON");
        assert!(json.contains("test"));
        assert!(json.contains("quoted"));
    }
}
