use std::io::Write;
use word_tally::{Count, Input, Io, Options, Processing, WordTally};

const API_EXAMPLE_TEXT: &str = "The quick brown fox jumps over the lazy dog";
const EXPECTED_API_WORD_COUNT: Count = 9;
const EXPECTED_API_UNIQ_COUNT: Count = 8;

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
    let options = Options::default()
        .with_io(Io::Streamed)
        .with_processing(Processing::Sequential);

    let mut temp_file = tempfile::NamedTempFile::new().unwrap();
    std::io::Write::write_all(&mut temp_file, API_EXAMPLE_TEXT.as_bytes()).unwrap();

    let input = Input::new(temp_file.path().to_str().unwrap(), options.io())
        .expect("Failed to create Input");

    let word_tally = WordTally::new(&input, &options).expect("Failed to create WordTally");
    verify_api_example_tally(&word_tally);
}

#[test]
fn test_api_buffered_sequential() {
    let options = Options::default()
        .with_io(Io::Buffered)
        .with_processing(Processing::Sequential);

    // Create a temporary file with our text
    let mut temp_file = tempfile::NamedTempFile::new().unwrap();
    std::io::Write::write_all(&mut temp_file, API_EXAMPLE_TEXT.as_bytes()).unwrap();

    let input = Input::new(temp_file.path().to_str().unwrap(), options.io())
        .expect("Failed to create Input");

    let word_tally = WordTally::new(&input, &options).expect("Failed to create WordTally");
    verify_api_example_tally(&word_tally);
}

#[test]
fn test_api_streamed_parallel() {
    let options = Options::default()
        .with_io(Io::Streamed)
        .with_processing(Processing::Parallel);

    let mut temp_file = tempfile::NamedTempFile::new().unwrap();
    std::io::Write::write_all(&mut temp_file, API_EXAMPLE_TEXT.as_bytes()).unwrap();

    let input = Input::new(temp_file.path().to_str().unwrap(), options.io())
        .expect("Failed to create Input");

    let word_tally = WordTally::new(&input, &options).expect("Failed to create WordTally");
    verify_api_example_tally(&word_tally);
}

#[test]
fn test_api_buffered_parallel() {
    let options = Options::default()
        .with_io(Io::Buffered)
        .with_processing(Processing::Parallel);

    let mut temp_file = tempfile::NamedTempFile::new().unwrap();
    Write::write_all(&mut temp_file, API_EXAMPLE_TEXT.as_bytes()).unwrap();

    let input = Input::new(temp_file.path().to_str().unwrap(), options.io())
        .expect("Failed to create Input");

    let word_tally = WordTally::new(&input, &options).expect("Failed to create WordTally");
    verify_api_example_tally(&word_tally);
}

#[test]
fn test_api_memory_mapped() {
    let temp_dir = tempfile::tempdir().unwrap();
    let file_path = temp_dir.path().join("example.txt");
    std::fs::write(&file_path, API_EXAMPLE_TEXT).unwrap();

    let options = Options::default()
        .with_io(Io::MemoryMapped)
        .with_processing(Processing::Sequential);

    let input =
        Input::new(file_path.to_str().unwrap(), Io::MemoryMapped).expect("Failed to create Input");

    let word_tally = WordTally::new(&input, &options).expect("Failed to create WordTally");
    verify_api_example_tally(&word_tally);
}

#[test]
fn test_api_memory_mapped_parallel() {
    let temp_dir = tempfile::tempdir().unwrap();
    let file_path = temp_dir.path().join("example_parallel.txt");
    std::fs::write(&file_path, API_EXAMPLE_TEXT).unwrap();

    let options = Options::default()
        .with_io(Io::MemoryMapped)
        .with_processing(Processing::Parallel);

    let input =
        Input::new(file_path.to_str().unwrap(), Io::MemoryMapped).expect("Failed to create Input");

    let word_tally = WordTally::new(&input, &options).expect("Failed to create WordTally");
    verify_api_example_tally(&word_tally);
}

#[test]
fn test_api_comprehensive_example() {
    let text = "Example text for API demonstration";
    let temp_dir = tempfile::tempdir().unwrap();
    let file_path = temp_dir.path().join("test_example.txt");
    std::fs::write(&file_path, text).unwrap();
    let file_path_str = file_path.to_str().unwrap();

    let options_streamed_seq = Options::default()
        .with_io(Io::Streamed)
        .with_processing(Processing::Sequential);

    let options_buffered_seq = Options::default()
        .with_io(Io::Buffered)
        .with_processing(Processing::Sequential);

    let options_streamed_par = Options::default()
        .with_io(Io::Streamed)
        .with_processing(Processing::Parallel);

    let options_buffered_par = Options::default()
        .with_io(Io::Buffered)
        .with_processing(Processing::Parallel);

    let options_mmap_seq = Options::default()
        .with_io(Io::MemoryMapped)
        .with_processing(Processing::Sequential);

    let options_mmap_par = Options::default()
        .with_io(Io::MemoryMapped)
        .with_processing(Processing::Parallel);

    let input_streamed_seq = Input::new(file_path_str, Io::Streamed)
        .expect("Failed to create streamed sequential input");

    let input_buffered_seq = Input::new(file_path_str, Io::Buffered)
        .expect("Failed to create buffered sequential input");

    let input_streamed_par =
        Input::new(file_path_str, Io::Streamed).expect("Failed to create streamed parallel input");

    let input_buffered_par =
        Input::new(file_path_str, Io::Buffered).expect("Failed to create buffered parallel input");

    let input_mmap_seq = Input::new(file_path_str, Io::MemoryMapped)
        .expect("Failed to create memory-mapped sequential input");

    let input_mmap_par = Input::new(file_path_str, Io::MemoryMapped)
        .expect("Failed to create memory-mapped parallel input");

    let count_checks = [
        WordTally::new(&input_streamed_seq, &options_streamed_seq)
            .expect("Failed with streamed sequential"),
        WordTally::new(&input_buffered_seq, &options_buffered_seq)
            .expect("Failed with buffered sequential"),
        WordTally::new(&input_streamed_par, &options_streamed_par)
            .expect("Failed with streamed parallel"),
        WordTally::new(&input_buffered_par, &options_buffered_par)
            .expect("Failed with buffered parallel"),
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

    let options_seq = Options::default()
        .with_io(Io::Bytes)
        .with_processing(Processing::Sequential);

    let options_par = Options::default()
        .with_io(Io::Bytes)
        .with_processing(Processing::Parallel);

    let seq_tally =
        WordTally::new(&bytes_input, &options_seq).expect("Failed with bytes sequential");
    let par_tally = WordTally::new(&bytes_input, &options_par).expect("Failed with bytes parallel");

    verify_api_example_tally(&seq_tally);
    verify_api_example_tally(&par_tally);

    assert_eq!(seq_tally.count(), par_tally.count());
    assert_eq!(seq_tally.uniq_count(), par_tally.uniq_count());
}
