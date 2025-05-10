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

use std::io::Cursor;
use word_tally::{Count, Io, Options, Processing, WordTally};

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
    let tally =
        WordTally::new(Cursor::new(TEST_TEXT), &options).expect("Failed to create WordTally");

    verify_tally(&tally);
}

#[test]
fn test_streamed_parallel() {
    let options = make_options(Io::Streamed, Processing::Parallel);
    let tally =
        WordTally::new(Cursor::new(TEST_TEXT), &options).expect("Failed to create WordTally");

    verify_tally(&tally);
}

#[test]
fn test_buffered_sequential() {
    let options = make_options(Io::Buffered, Processing::Sequential);
    let tally =
        WordTally::new(Cursor::new(TEST_TEXT), &options).expect("Failed to create WordTally");

    verify_tally(&tally);
}

#[test]
fn test_buffered_parallel() {
    let options = make_options(Io::Buffered, Processing::Parallel);
    let tally =
        WordTally::new(Cursor::new(TEST_TEXT), &options).expect("Failed to create WordTally");

    verify_tally(&tally);
}

#[test]
fn test_new_with_io_combinations() {
    use std::io::Cursor;

    // Test streamed and buffered I/O with both sequential and parallel processing
    let io_strategies = [Io::Streamed, Io::Buffered];
    let processing_strategies = [Processing::Sequential, Processing::Parallel];

    for &io in &io_strategies {
        for &processing in &processing_strategies {
            let options = make_options(io, processing);
            let tally = WordTally::new(Cursor::new(TEST_TEXT), &options)
                .unwrap_or_else(|_| panic!("new failed with {io:?}/{processing:?}"));

            verify_tally(&tally);
        }
    }

    // Memory-mapped I/O should error with new() since it requires a File
    let mmap_options = make_options(Io::MemoryMapped, Processing::Sequential);
    let result = WordTally::new(Cursor::new(TEST_TEXT), &mmap_options);
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Memory-mapped I/O requires a file input. Use a file path instead of stdin.")
    );

    let mmap_parallel_options = make_options(Io::MemoryMapped, Processing::Parallel);
    let result = WordTally::new(Cursor::new(TEST_TEXT), &mmap_parallel_options);
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Memory-mapped I/O requires a file input. Use a file path instead of stdin.")
    );
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
    let sequential_options = make_options(Io::Buffered, Processing::Sequential);
    let parallel_options = make_options(Io::Buffered, Processing::Parallel);

    let sequential_tally = WordTally::new(Cursor::new(LARGE_TEST_TEXT), &sequential_options)
        .expect("Failed to create sequential WordTally");
    let parallel_tally = WordTally::new(Cursor::new(LARGE_TEST_TEXT), &parallel_options)
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
    use std::env::temp_dir;
    use std::fs::File;
    use std::io::Write;

    // Create a temporary file with test data
    let mut temp_path = temp_dir();
    temp_path.push("word_tally_test.txt");

    {
        let mut file = File::create(&temp_path).expect("Failed to create temp file");
        file.write_all(TEST_TEXT.as_bytes())
            .expect("Failed to write test data");
    }

    let file = File::open(&temp_path).expect("Failed to open temp file");

    // Test sequential memory-mapped I/O
    let mmap_sequential_options = make_options(Io::MemoryMapped, Processing::Sequential);
    let mmap_tally = WordTally::from_file(&file, &mmap_sequential_options)
        .expect("Failed to process file with memory mapping");

    verify_tally(&mmap_tally);

    // Open the file again for parallel test
    let file = File::open(&temp_path).expect("Failed to open temp file");

    // Test parallel memory-mapped I/O
    let mmap_parallel_options = make_options(Io::MemoryMapped, Processing::Parallel);
    let mmap_parallel_tally = WordTally::from_file(&file, &mmap_parallel_options)
        .expect("Failed to process file with memory mapping");

    verify_tally(&mmap_parallel_tally);

    // Clean up
    std::fs::remove_file(temp_path).expect("Failed to remove temp file");
}

// Test error handling for try_from_file
#[test]
fn test_nonexistent_file_handling() {
    use std::fs::File;
    use std::path::PathBuf;

    let path = PathBuf::from("/nonexistent/file/path");
    let file_result = File::open(&path);
    assert!(file_result.is_err());

    if let Ok(file) = file_result {
        let options = make_options(Io::MemoryMapped, Processing::Sequential);
        let result = WordTally::from_file(&file, &options);
        assert!(result.is_err());
    }
}

#[test]
fn test_from_file_with_all_io_strategies() {
    use std::env::temp_dir;
    use std::fs::File;
    use std::io::Write;

    let mut temp_path = temp_dir();
    temp_path.push("word_tally_io_test.txt");

    {
        let mut file = File::create(&temp_path).expect("Failed to create temp file");
        file.write_all(TEST_TEXT.as_bytes())
            .expect("Failed to write test data");
    }

    // Test all combinations of I/O and processing strategies
    let io_strategies = [Io::Streamed, Io::Buffered, Io::MemoryMapped];
    let processing_strategies = [Processing::Sequential, Processing::Parallel];

    for &io in &io_strategies {
        for &processing in &processing_strategies {
            let file = File::open(&temp_path).expect("Failed to open temp file");
            let options = make_options(io, processing);
            let tally = WordTally::from_file(&file, &options)
                .unwrap_or_else(|_| panic!("from_file failed with {io:?}/{processing:?}"));

            verify_tally(&tally);
        }
    }

    std::fs::remove_file(temp_path).expect("Failed to remove temp file");
}
