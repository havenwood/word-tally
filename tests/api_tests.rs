//! Tests for the public API.

use std::io::Write;

use word_tally::{Count, Input, Io, Options, WordTally};

const API_EXAMPLE_TEXT: &str = "I taste a liquor never brewed";
const EXPECTED_API_WORD_COUNT: Count = 6;
const EXPECTED_API_UNIQ_COUNT: Count = 6;

fn verify_api_example_tally(tally: &WordTally) {
    assert_eq!(
        tally.count(),
        EXPECTED_API_WORD_COUNT,
        "Total word count mismatch"
    );
    assert_eq!(
        tally.uniq_count(),
        EXPECTED_API_UNIQ_COUNT,
        "Unique word count mismatch"
    );
}

#[test]
fn test_api_basic_functionality() {
    let options = Options::default();
    let mut temp_file = tempfile::NamedTempFile::new().expect("create temp file");
    Write::write_all(&mut temp_file, API_EXAMPLE_TEXT.as_bytes()).expect("process test");

    let input = Input::new(temp_file.path(), options.io()).expect("create input");
    let word_tally = WordTally::new(&input, &options).expect("create word tally");

    verify_api_example_tally(&word_tally);
}

#[test]
fn test_from_bytes_api() {
    let bytes_input = Input::from(API_EXAMPLE_TEXT.as_bytes());
    let options = Options::default().with_io(Io::ParallelBytes);
    let tally = WordTally::new(&bytes_input, &options).expect("process bytes");

    verify_api_example_tally(&tally);
}
