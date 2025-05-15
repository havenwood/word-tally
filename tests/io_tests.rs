// Tests for the I/O and Processing strategy implementation
//
// This test file specifically tests all 6 combinations of I/O and processing strategies:
// - Streamed + Sequential
// - Streamed + Parallel
// - Buffered + Sequential
// - Buffered + Parallel
// - MemoryMapped + Sequential
// - MemoryMapped + Parallel
//
// Each strategy is tested to verify they all produce identical results.

use std::io::Write;
use word_tally::{Count, Input, Io, Options, Performance, Processing, WordTally};

// Test data that's small enough for tests but has multiple lines and words
const TEST_TEXT: &str = "The quick brown fox
jumps over the lazy dog
Pack my box with five dozen liquor jugs";

// Unique words (note: "the" appears twice)
const EXPECTED_WORD_COUNT: Count = 16;
// Total words including duplicates
const EXPECTED_TOTAL_COUNT: Count = 17;

// Helper to create options with specific I/O and processing strategy
fn make_options(io: Io, processing: Processing) -> Options {
    Options::default().with_io(io).with_processing(processing)
}

// Helper to verify tally results
fn verify_tally(tally: &WordTally<'_>) {
    // Print words on failure for debugging purposes
    if tally.count() != EXPECTED_TOTAL_COUNT || tally.uniq_count() != EXPECTED_WORD_COUNT {
        println!("Words found: {:?}", tally.tally());
    }

    assert_eq!(
        tally.count(),
        EXPECTED_TOTAL_COUNT,
        "Total word count mismatch"
    );
    assert_eq!(
        tally.uniq_count(),
        EXPECTED_WORD_COUNT,
        "Unique word count mismatch"
    );
}

#[test]
fn test_streamed_sequential() {
    let options = make_options(Io::Streamed, Processing::Sequential);

    let mut temp_file = tempfile::NamedTempFile::new().unwrap();
    Write::write_all(&mut temp_file, TEST_TEXT.as_bytes()).unwrap();

    let input = Input::new(temp_file.path().to_str().unwrap(), options.io())
        .expect("Failed to create Input");

    let tally = WordTally::new(&input, &options).expect("Failed to create WordTally");

    verify_tally(&tally);
}

#[test]
fn test_streamed_parallel() {
    let options = make_options(Io::Streamed, Processing::Parallel);

    let mut temp_file = tempfile::NamedTempFile::new().unwrap();
    Write::write_all(&mut temp_file, TEST_TEXT.as_bytes()).unwrap();

    let input = Input::new(temp_file.path().to_str().unwrap(), options.io())
        .expect("Failed to create Input");

    let tally = WordTally::new(&input, &options).expect("Failed to create WordTally");

    verify_tally(&tally);
}

#[test]
fn test_buffered_sequential() {
    let options = make_options(Io::Buffered, Processing::Sequential);

    let mut temp_file = tempfile::NamedTempFile::new().unwrap();
    Write::write_all(&mut temp_file, TEST_TEXT.as_bytes()).unwrap();

    let input = Input::new(temp_file.path().to_str().unwrap(), options.io())
        .expect("Failed to create Input");

    let tally = WordTally::new(&input, &options).expect("Failed to create WordTally");

    verify_tally(&tally);
}

#[test]
fn test_buffered_parallel() {
    let options = make_options(Io::Buffered, Processing::Parallel);

    let mut temp_file = tempfile::NamedTempFile::new().unwrap();
    Write::write_all(&mut temp_file, TEST_TEXT.as_bytes()).unwrap();

    let input = Input::new(temp_file.path().to_str().unwrap(), options.io())
        .expect("Failed to create Input");

    let tally = WordTally::new(&input, &options).expect("Failed to create WordTally");

    verify_tally(&tally);
}

#[test]
fn test_new_with_io_combinations() {
    let mut temp_file = tempfile::NamedTempFile::new().unwrap();
    Write::write_all(&mut temp_file, TEST_TEXT.as_bytes()).unwrap();
    let file_path = temp_file.path().to_str().unwrap();

    let io_strategies = [Io::Streamed, Io::Buffered, Io::MemoryMapped];
    let processing_strategies = [Processing::Sequential, Processing::Parallel];

    for &io in &io_strategies {
        for &processing in &processing_strategies {
            let options = make_options(io, processing);
            let input = Input::new(file_path, io)
                .unwrap_or_else(|_| panic!("input creation failed with {io:?}/{processing:?}"));

            let tally = WordTally::new(&input, &options)
                .unwrap_or_else(|_| panic!("new failed with {io:?}/{processing:?}"));

            verify_tally(&tally);
        }
    }
}

// Test with a larger dataset to ensure chunking works properly in parallel mode
const LARGE_TEST_TEXT: &str = "
Lorem ipsum dolor sit amet, consectetur adipiscing elit.
Suspendisse sodales felis in arcu scelerisque, at finibus ante fermentum.
Cras eget lacus vel neque condimentum commodo eget ut quam.
Etiam venenatis pharetra diam, eu volutpat dui.
Mauris consectetur nisi quis pretium efficitur.
Nulla facilisi. Fusce ultricies dictum massa, nec ultrices turpis volutpat a.
Sed a tellus sed sapien congue sodales.
Aenean bibendum, leo vel faucibus auctor, erat dui maximus tortor, et imperdiet arcu ex ut enim.
Nulla sagittis orci quis urna egestas, eu ultricies massa tristique.
Vestibulum semper tortor a lorem tempus, ac vulputate arcu ultricies.
Nam suscipit varius turpis, vel interdum sem ultrices vel.
Morbi tincidunt elit a tellus ultrices, vel placerat turpis ultricies.
Fusce faucibus nisi id velit scelerisque, sed pulvinar est cursus.
Nulla facilisi. Proin commodo leo vel condimentum molestie.
Aliquam ultrices finibus varius. Aliquam erat volutpat.
Vivamus eget elit a tortor convallis ultrices.
Nullam efficitur, mi at dapibus tincidunt, risus orci vulputate lacus, et vehicula lorem nunc vel dui.
";

// Test with a large text to ensure parallel processing works correctly
#[test]
fn test_parallel_processing_with_large_text() {
    let mut temp_file = tempfile::NamedTempFile::new().unwrap();
    Write::write_all(&mut temp_file, LARGE_TEST_TEXT.as_bytes()).unwrap();
    let file_path = temp_file.path().to_str().unwrap();

    let sequential_options = make_options(Io::Buffered, Processing::Sequential);
    let parallel_options = make_options(Io::Buffered, Processing::Parallel);

    let sequential_input =
        Input::new(file_path, sequential_options.io()).expect("Failed to create sequential input");
    let parallel_input =
        Input::new(file_path, parallel_options.io()).expect("Failed to create parallel input");

    let sequential_tally = WordTally::new(&sequential_input, &sequential_options)
        .expect("Failed to create sequential WordTally");
    let parallel_tally = WordTally::new(&parallel_input, &parallel_options)
        .expect("Failed to create parallel WordTally");

    // Both should produce identical results
    assert_eq!(sequential_tally.count(), parallel_tally.count());
    assert_eq!(sequential_tally.uniq_count(), parallel_tally.uniq_count());

    // Check that tallies contain the same words with the same counts
    // but don't require exact same order due to parallel sorting being unstable
    let sequential_words: Vec<_> = sequential_tally.tally().to_vec();
    let parallel_words: Vec<_> = parallel_tally.tally().to_vec();

    // Convert to HashMaps for content comparison rather than order comparison
    let seq_map: std::collections::HashMap<_, _> = sequential_words.into_iter().collect();
    let par_map: std::collections::HashMap<_, _> = parallel_words.into_iter().collect();

    assert_eq!(seq_map, par_map);
}

// Test with file-backed memory mapped I/O
#[test]
fn test_memory_mapped_with_real_file() {
    // Create a temporary file with test data
    let mut temp_file = tempfile::NamedTempFile::new().unwrap();
    Write::write_all(&mut temp_file, TEST_TEXT.as_bytes()).unwrap();
    let file_path = temp_file.path().to_str().unwrap();

    // Test sequential memory-mapped I/O
    let mmap_sequential_options = make_options(Io::MemoryMapped, Processing::Sequential);
    let sequential_input = Input::new(file_path, mmap_sequential_options.io())
        .expect("Failed to create sequential memory-mapped input");

    let mmap_tally = WordTally::new(&sequential_input, &mmap_sequential_options)
        .expect("Failed to process file with memory mapping");

    verify_tally(&mmap_tally);

    // Test parallel memory-mapped I/O
    let mmap_parallel_options = make_options(Io::MemoryMapped, Processing::Parallel);
    let parallel_input = Input::new(file_path, mmap_parallel_options.io())
        .expect("Failed to create parallel memory-mapped input");

    let mmap_parallel_tally = WordTally::new(&parallel_input, &mmap_parallel_options)
        .expect("Failed to process file with memory mapping");

    verify_tally(&mmap_parallel_tally);
}

#[test]
fn test_read_trait_with_all_io_strategies() {
    use std::io::Read;

    let temp_dir = tempfile::tempdir().unwrap();
    let file_path = temp_dir.path().join("test_io.txt");
    std::fs::write(&file_path, TEST_TEXT).unwrap();

    let file_input = Input::new(&file_path, Io::Streamed).unwrap();
    let mmap_input = Input::new(&file_path, Io::MemoryMapped).unwrap();
    let bytes_input = Input::from_bytes(TEST_TEXT);
    // Note: We'll skip stdin since it would block waiting for input

    let test_cases = [file_input, mmap_input, bytes_input];

    for input in &test_cases {
        let mut reader = input.reader().expect("Failed to create reader");
        let mut content = String::new();
        reader
            .read_to_string(&mut content)
            .expect("Failed to read content");

        assert_eq!(content.trim(), TEST_TEXT.trim());
    }
}

#[test]
fn test_bytes_input() {
    let processing_strategies = [Processing::Sequential, Processing::Parallel];

    for &processing in &processing_strategies {
        let options = make_options(Io::Bytes, processing);
        let input = Input::from_bytes(TEST_TEXT);

        assert_eq!(input.source(), "<bytes>");
        assert_eq!(input.size(), Some(TEST_TEXT.len()));

        let tally = WordTally::new(&input, &options).expect("Failed to create WordTally");
        verify_tally(&tally);
    }
}

#[test]
fn test_bytes_io_with_input_new() {
    let result = Input::new(TEST_TEXT, Io::Bytes);
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("use `Input::from_bytes()`"));
}

// Test error handling for non-existent files
#[test]
fn test_nonexistent_file_handling() {
    use std::fs::File;
    use std::path::PathBuf;

    let path = PathBuf::from("/nonexistent/file/path");
    let file_result = File::open(&path);
    assert!(file_result.is_err());
}

#[test]
fn test_new_with_all_io_strategies() {
    // Create a temporary file with test data
    let mut temp_file = tempfile::NamedTempFile::new().unwrap();
    Write::write_all(&mut temp_file, TEST_TEXT.as_bytes()).unwrap();
    let file_path = temp_file.path().to_str().unwrap();

    // Test all combinations of I/O and processing strategies
    let io_strategies = [Io::Streamed, Io::Buffered, Io::MemoryMapped];
    let processing_strategies = [Processing::Sequential, Processing::Parallel];

    for &io in &io_strategies {
        for &processing in &processing_strategies {
            let options = make_options(io, processing);
            let input = Input::new(file_path, io)
                .unwrap_or_else(|_| panic!("input creation failed with {io:?}/{processing:?}"));

            let tally = WordTally::new(&input, &options)
                .unwrap_or_else(|_| panic!("new() failed with {io:?}/{processing:?}"));

            verify_tally(&tally);
        }
    }
}

#[test]
fn test_utf8_boundary_handling() {
    let test_text = "æ æ æ æ";

    let mut temp_file = tempfile::NamedTempFile::new().unwrap();
    Write::write_all(&mut temp_file, test_text.as_bytes()).unwrap();
    let file_path = temp_file.path().to_str().unwrap();

    // Use a small chunk size directly in the performance settings
    let small_chunk_size = 32;
    let performance = Performance::default().with_chunk_size(small_chunk_size);

    let options = Options::default()
        .with_io(Io::MemoryMapped)
        .with_processing(Processing::Parallel)
        .with_performance(performance);

    let input = Input::new(file_path, options.io()).expect("Failed to create input");
    let tally = WordTally::new(&input, &options).expect("WordTally creation failed");

    assert!(
        tally
            .tally()
            .iter()
            .map(|(word, _)| word)
            .any(|word| word == &"æ".into()),
        "Missing 'æ' in results"
    );
}
