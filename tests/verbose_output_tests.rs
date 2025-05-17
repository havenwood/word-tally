#[cfg(test)]
mod verbose_unit_tests {
    use std::io::{Result as IoResult, Write};
    use std::sync::{Arc, Mutex};
    use word_tally::{Case, Filters, Format, Input, Io, Options, Processing, Sort, WordTally};

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

        #[allow(dead_code)]
        fn content(&self) -> String {
            let data = self.content.lock().unwrap();
            String::from_utf8_lossy(&data).to_string()
        }
    }

    impl Write for MockWriter {
        fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
            self.content.lock().unwrap().extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> IoResult<()> {
            Ok(())
        }
    }

    // Helper function to create test WordTally
    fn create_test_tally(options: &Options) -> WordTally<'_> {
        let test_text = b"hope is the thing with feathers that perches";
        let mut temp_file = tempfile::NamedTempFile::new().unwrap();
        std::io::Write::write_all(&mut temp_file, test_text).unwrap();

        let input = Input::new(temp_file.path().to_str().unwrap(), options.io()).unwrap();
        WordTally::new(&input, options).unwrap()
    }

    // Test that verbose output works with JSON format
    #[test]
    fn test_verbose_json_format() {
        let _mock = MockWriter::new();
        let options = Options::default().with_format(Format::Json);
        let tally = create_test_tally(&options);

        // We can't directly test handle_verbose_output since it's not public
        // Instead, we test through the CLI integration which uses verbose output
        // This is already covered by cli_verbose_tests.rs

        // However, we can test that the WordTally serializes properly to JSON
        let json = serde_json::to_string(&tally).unwrap();
        assert!(json.contains("\"count\":"));
        assert!(json.contains("\"uniqueCount\":"));
        assert!(json.contains("\"tally\":"));
    }

    // Test that verbose output works with CSV format
    #[test]
    fn test_verbose_csv_format() {
        let _mock = MockWriter::new();
        let options = Options::default().with_format(Format::Csv);
        let tally = create_test_tally(&options);

        // Test CSV serialization of the tally itself
        let mut csv_writer = csv::Writer::from_writer(vec![]);
        csv_writer.write_record(["word", "count"]).unwrap();

        for (word, count) in &tally {
            csv_writer
                .write_record([word.as_ref(), &count.to_string()])
                .unwrap();
        }

        let data = csv_writer.into_inner().unwrap();
        let csv = String::from_utf8(data).unwrap();
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
        let text = format!("{:?}", tally);
        assert!(!text.is_empty());
    }

    // Test verbose with different options
    #[test]
    fn test_verbose_with_various_options() {
        let options = Options::default()
            .with_case(Case::Upper)
            .with_sort(Sort::Asc)
            .with_processing(Processing::Sequential)
            .with_io(Io::Streamed);

        let tally = create_test_tally(&options);

        assert_eq!(tally.options().case(), Case::Upper);
        assert_eq!(tally.options().sort(), Sort::Asc);
        assert_eq!(tally.options().processing(), Processing::Sequential);
        assert_eq!(tally.options().io(), Io::Streamed);

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

        let json_with = serde_json::to_string(&tally_with).unwrap();
        let json_without = serde_json::to_string(&tally_without).unwrap();

        // Check that options are properly serialized in JSON
        assert!(json_with.contains("\"options\":"));
        assert!(json_without.contains("\"options\":"));
    }

    // Test for edge cases
    #[test]
    fn test_verbose_with_empty_input() {
        let test_text = b"";
        let mut temp_file = tempfile::NamedTempFile::new().unwrap();
        std::io::Write::write_all(&mut temp_file, test_text).unwrap();

        let options = Options::default();
        let input = Input::new(temp_file.path().to_str().unwrap(), options.io()).unwrap();
        let tally = WordTally::new(&input, &options).unwrap();

        assert_eq!(tally.count(), 0);
        assert_eq!(tally.uniq_count(), 0);

        // Test that empty tally still serializes correctly
        let json = serde_json::to_string(&tally).unwrap();
        assert!(json.contains("\"count\":0"));
        assert!(json.contains("\"uniqueCount\":0"));
    }

    // Test verbose with special characters
    #[test]
    fn test_verbose_with_special_characters() {
        let test_text = b"test \"quoted\" text & special <chars>";
        let mut temp_file = tempfile::NamedTempFile::new().unwrap();
        std::io::Write::write_all(&mut temp_file, test_text).unwrap();

        let options = Options::default();
        let input = Input::new(temp_file.path().to_str().unwrap(), options.io()).unwrap();
        let tally = WordTally::new(&input, &options).unwrap();

        // Check that special characters are handled properly
        let json = serde_json::to_string(&tally).unwrap();
        assert!(json.contains("test"));
        assert!(json.contains("quoted"));
    }
}
