//! Tests for the public API.

use std::io::Write;
use word_tally::{Count, Input, Io, Options, WordTally};

const API_EXAMPLE_TEXT: &str = "I taste a liquor never brewed";
const EXPECTED_API_WORD_COUNT: Count = 6;
const EXPECTED_API_UNIQ_COUNT: Count = 6;

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
    let options = Options::default().with_io(Io::ParallelStream);

    let mut temp_file = tempfile::NamedTempFile::new().expect("create temp file");
    std::io::Write::write_all(&mut temp_file, API_EXAMPLE_TEXT.as_bytes()).expect("process test");

    let input = Input::new(
        temp_file.path().to_str().expect("temp file path"),
        options.io(),
    )
    .expect("Failed to create Input");

    let word_tally = WordTally::new(&input, &options).expect("Failed to create WordTally");
    verify_api_example_tally(&word_tally);
}

#[test]
fn test_api_in_memory_sequential() {
    let options = Options::default().with_io(Io::ParallelInMemory);

    // Create a temporary file with our text
    let mut temp_file = tempfile::NamedTempFile::new().expect("create temp file");
    std::io::Write::write_all(&mut temp_file, API_EXAMPLE_TEXT.as_bytes()).expect("process test");

    let input = Input::new(
        temp_file.path().to_str().expect("temp file path"),
        options.io(),
    )
    .expect("Failed to create Input");

    let word_tally = WordTally::new(&input, &options).expect("Failed to create WordTally");
    verify_api_example_tally(&word_tally);
}

#[test]
fn test_api_streamed_parallel() {
    let options = Options::default().with_io(Io::ParallelStream);

    let mut temp_file = tempfile::NamedTempFile::new().expect("create temp file");
    std::io::Write::write_all(&mut temp_file, API_EXAMPLE_TEXT.as_bytes()).expect("process test");

    let input = Input::new(
        temp_file.path().to_str().expect("temp file path"),
        options.io(),
    )
    .expect("Failed to create Input");

    let word_tally = WordTally::new(&input, &options).expect("Failed to create WordTally");
    verify_api_example_tally(&word_tally);
}

#[test]
fn test_api_in_memory_parallel() {
    let options = Options::default().with_io(Io::ParallelInMemory);

    let mut temp_file = tempfile::NamedTempFile::new().expect("create temp file");
    Write::write_all(&mut temp_file, API_EXAMPLE_TEXT.as_bytes()).expect("process test");

    let input = Input::new(
        temp_file.path().to_str().expect("temp file path"),
        options.io(),
    )
    .expect("Failed to create Input");

    let word_tally = WordTally::new(&input, &options).expect("Failed to create WordTally");
    verify_api_example_tally(&word_tally);
}

#[test]
fn test_api_memory_mapped() {
    let temp_dir = tempfile::tempdir().expect("process test");
    let file_path = temp_dir.path().join("example.txt");
    std::fs::write(&file_path, API_EXAMPLE_TEXT).expect("process test");

    let options = Options::default().with_io(Io::ParallelMmap);

    let input = Input::new(
        file_path.to_str().expect("create test input"),
        Io::ParallelMmap,
    )
    .expect("Failed to create Input");

    let word_tally = WordTally::new(&input, &options).expect("Failed to create WordTally");
    verify_api_example_tally(&word_tally);
}

#[test]
fn test_api_memory_mapped_parallel() {
    let temp_dir = tempfile::tempdir().expect("process test");
    let file_path = temp_dir.path().join("example_parallel.txt");
    std::fs::write(&file_path, API_EXAMPLE_TEXT).expect("process test");

    let options = Options::default().with_io(Io::ParallelMmap);

    let input = Input::new(
        file_path.to_str().expect("create test input"),
        Io::ParallelMmap,
    )
    .expect("Failed to create Input");

    let word_tally = WordTally::new(&input, &options).expect("Failed to create WordTally");
    verify_api_example_tally(&word_tally);
}

#[test]
fn test_api_comprehensive_example() {
    let text = "Example text for API demonstration";
    let temp_dir = tempfile::tempdir().expect("process test");
    let file_path = temp_dir.path().join("test_example.txt");
    std::fs::write(&file_path, text).expect("process test");
    let file_path_str = file_path.to_str().expect("process test");

    let options_streamed_seq = Options::default().with_io(Io::ParallelStream);

    let options_in_memory_seq = Options::default().with_io(Io::ParallelInMemory);

    let options_streamed_par = Options::default().with_io(Io::ParallelStream);

    let options_in_memory_par = Options::default().with_io(Io::ParallelInMemory);

    let options_mmap_seq = Options::default().with_io(Io::ParallelMmap);

    let options_mmap_par = Options::default().with_io(Io::ParallelMmap);

    let input_streamed_seq = Input::new(file_path_str, Io::ParallelStream)
        .expect("Failed to create streamed sequential input");

    let input_in_memory_seq = Input::new(file_path_str, Io::ParallelInMemory)
        .expect("Failed to create in-memory sequential input");

    let input_streamed_par = Input::new(file_path_str, Io::ParallelStream)
        .expect("Failed to create streamed parallel input");

    let input_in_memory_par = Input::new(file_path_str, Io::ParallelInMemory)
        .expect("Failed to create in-memory parallel input");

    let input_mmap_seq = Input::new(file_path_str, Io::ParallelMmap)
        .expect("Failed to create memory-mapped sequential input");

    let input_mmap_par = Input::new(file_path_str, Io::ParallelMmap)
        .expect("Failed to create memory-mapped parallel input");

    let count_checks = [
        WordTally::new(&input_streamed_seq, &options_streamed_seq)
            .expect("Failed with streamed sequential"),
        WordTally::new(&input_in_memory_seq, &options_in_memory_seq)
            .expect("Failed with in-memory sequential"),
        WordTally::new(&input_streamed_par, &options_streamed_par)
            .expect("Failed with streamed parallel"),
        WordTally::new(&input_in_memory_par, &options_in_memory_par)
            .expect("Failed with in-memory parallel"),
        WordTally::new(&input_mmap_seq, &options_mmap_seq)
            .expect("Failed with memory-mapped sequential"),
        WordTally::new(&input_mmap_par, &options_mmap_par)
            .expect("Failed with memory-mapped parallel"),
    ];

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
}

#[test]
fn test_from_bytes_api() {
    let bytes_input = Input::from_bytes(API_EXAMPLE_TEXT);

    let options_seq = Options::default().with_io(Io::ParallelBytes);

    let options_par = Options::default().with_io(Io::ParallelBytes);

    let seq_tally =
        WordTally::new(&bytes_input, &options_seq).expect("Failed with bytes sequential");
    let par_tally = WordTally::new(&bytes_input, &options_par).expect("Failed with bytes parallel");

    verify_api_example_tally(&seq_tally);
    verify_api_example_tally(&par_tally);

    assert_eq!(seq_tally.count(), par_tally.count());
    assert_eq!(seq_tally.uniq_count(), par_tally.uniq_count());
}
