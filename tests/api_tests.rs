//! Tests for the public API.

use std::io::Write;

use word_tally::{Count, Io, Options, Reader, TallyMap, View, WordTally};

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

    let reader = Reader::try_from(temp_file.path()).expect("create reader");
    let tally_map = TallyMap::from_reader(&reader, &options).expect("create tally map");
    let word_tally = WordTally::from_tally_map(tally_map, &options);

    verify_api_example_tally(&word_tally);
}

#[test]
fn test_from_bytes_api() {
    let view = View::from(API_EXAMPLE_TEXT.as_bytes());
    let options = Options::default().with_io(Io::ParallelBytes);
    let tally_map = TallyMap::from_view(&view, &options).expect("create tally map");
    let tally = WordTally::from_tally_map(tally_map, &options);

    verify_api_example_tally(&tally);
}
