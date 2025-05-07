use std::fs::File;
use std::io::Cursor;
use std::sync::Arc;
use word_tally::{Io, Options, Processing, WordTally};

const API_EXAMPLE_TEXT: &str = "The quick brown fox jumps over the lazy dog";
const EXPECTED_API_WORD_COUNT: usize = 9;
const EXPECTED_API_UNIQ_COUNT: usize = 8;

fn verify_api_example_tally(tally: &WordTally<'_>) {
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
fn test_api_streamed_sequential() {
    let input = Cursor::new(API_EXAMPLE_TEXT);
    let options = Options::default()
        .with_io(Io::Streamed)
        .with_processing(Processing::Sequential);

    let word_tally = WordTally::new(input, &options);
    verify_api_example_tally(&word_tally);
}

#[test]
fn test_api_buffered_sequential() {
    let input = Cursor::new(API_EXAMPLE_TEXT);
    let options = Options::default()
        .with_io(Io::Buffered)
        .with_processing(Processing::Sequential);

    let word_tally = WordTally::new(input, &options);
    verify_api_example_tally(&word_tally);
}

#[test]
fn test_api_streamed_parallel() {
    let input = Cursor::new(API_EXAMPLE_TEXT);
    let options = Options::default()
        .with_io(Io::Streamed)
        .with_processing(Processing::Parallel);

    let word_tally = WordTally::new(input, &options);
    verify_api_example_tally(&word_tally);
}

#[test]
fn test_api_buffered_parallel() {
    let input = Cursor::new(API_EXAMPLE_TEXT);
    let options = Options::default()
        .with_io(Io::Buffered)
        .with_processing(Processing::Parallel);

    let word_tally = WordTally::new(input, &options);
    verify_api_example_tally(&word_tally);
}

#[test]
fn test_api_memory_mapped() {
    let temp_file = std::env::temp_dir().join("word_tally_api_mmap.txt");
    std::fs::write(&temp_file, API_EXAMPLE_TEXT).unwrap();

    let file = File::open(&temp_file).unwrap();
    let options = Options::default()
        .with_io(Io::MemoryMapped)
        .with_processing(Processing::Sequential);

    let word_tally = WordTally::try_from_file(file, &options).unwrap();
    verify_api_example_tally(&word_tally);

    std::fs::remove_file(temp_file).unwrap_or_default();
}

#[test]
fn test_api_memory_mapped_parallel() {
    let temp_file = std::env::temp_dir().join("word_tally_api_mmap_parallel.txt");
    std::fs::write(&temp_file, API_EXAMPLE_TEXT).unwrap();

    let file = File::open(&temp_file).unwrap();
    let options = Options::default()
        .with_io(Io::MemoryMapped)
        .with_processing(Processing::Parallel);

    let word_tally = WordTally::try_from_file(file, &options).unwrap();
    verify_api_example_tally(&word_tally);

    std::fs::remove_file(temp_file).unwrap_or_default();
}

#[test]
fn test_api_comprehensive_example() {
    // This test shows how to use the API with various options
    // It demonstrates all 6 combinations of I/O and processing
    let text = "Example text for API demonstration";

    let options_streamed_seq = Arc::new(
        Options::default()
            .with_io(Io::Streamed)
            .with_processing(Processing::Sequential),
    );

    let options_buffered_seq = Arc::new(
        Options::default()
            .with_io(Io::Buffered)
            .with_processing(Processing::Sequential),
    );

    let options_streamed_par = Arc::new(
        Options::default()
            .with_io(Io::Streamed)
            .with_processing(Processing::Parallel),
    );

    let options_buffered_par = Arc::new(
        Options::default()
            .with_io(Io::Buffered)
            .with_processing(Processing::Parallel),
    );

    // Each combination should produce the same result
    let count_checks = [
        WordTally::new(Cursor::new(text), &options_streamed_seq),
        WordTally::new(Cursor::new(text), &options_buffered_seq),
        WordTally::new(Cursor::new(text), &options_streamed_par),
        WordTally::new(Cursor::new(text), &options_buffered_par),
    ];

    // All strategies should produce the same results
    let expected_count = count_checks[0].count();
    let expected_uniq = count_checks[0].uniq_count();

    for (idx, tally) in count_checks.iter().enumerate() {
        assert_eq!(
            tally.count(),
            expected_count,
            "Total count mismatch at index {idx}"
        );
        assert_eq!(
            tally.uniq_count(),
            expected_uniq,
            "Unique count mismatch at index {idx}"
        );
    }

    // Test memory-mapped I/O separately since it requires a file
    let temp_file = std::env::temp_dir().join("word_tally_api_comprehensive.txt");
    std::fs::write(&temp_file, text).unwrap();

    // Memory-mapped variations
    let file1 = File::open(&temp_file).unwrap();
    let options_mmap_seq = Arc::new(
        Options::default()
            .with_io(Io::MemoryMapped)
            .with_processing(Processing::Sequential),
    );
    let mmap_seq = WordTally::try_from_file(file1, &options_mmap_seq).unwrap();

    let file2 = File::open(&temp_file).unwrap();
    let options_mmap_par = Arc::new(
        Options::default()
            .with_io(Io::MemoryMapped)
            .with_processing(Processing::Parallel),
    );
    let mmap_par = WordTally::try_from_file(file2, &options_mmap_par).unwrap();

    assert_eq!(
        mmap_seq.count(),
        expected_count,
        "Memory-mapped sequential count mismatch"
    );
    assert_eq!(
        mmap_seq.uniq_count(),
        expected_uniq,
        "Memory-mapped sequential unique count mismatch"
    );
    assert_eq!(
        mmap_par.count(),
        expected_count,
        "Memory-mapped parallel count mismatch"
    );
    assert_eq!(
        mmap_par.uniq_count(),
        expected_uniq,
        "Memory-mapped parallel unique count mismatch"
    );

    std::fs::remove_file(temp_file).unwrap_or_default();
}
