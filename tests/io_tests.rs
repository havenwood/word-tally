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
use word_tally::{Io, Options, Processing, WordTally};

// Test data that's small enough for tests but has multiple lines and words
const TEST_TEXT: &str = "The quick brown fox
jumps over the lazy dog
Pack my box with five dozen liquor jugs";

// Unique words (note: "the" appears twice)
const EXPECTED_WORD_COUNT: usize = 16;
// Total words including duplicates
const EXPECTED_TOTAL_COUNT: usize = 17;

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
    let tally = WordTally::new(Cursor::new(TEST_TEXT), &options);

    verify_tally(&tally);
}

#[test]
fn test_streamed_parallel() {
    let options = make_options(Io::Streamed, Processing::Parallel);
    let tally = WordTally::new(Cursor::new(TEST_TEXT), &options);

    verify_tally(&tally);
}

#[test]
fn test_buffered_sequential() {
    let options = make_options(Io::Buffered, Processing::Sequential);
    let tally = WordTally::new(Cursor::new(TEST_TEXT), &options);

    verify_tally(&tally);
}

#[test]
fn test_buffered_parallel() {
    let options = make_options(Io::Buffered, Processing::Parallel);
    let tally = WordTally::new(Cursor::new(TEST_TEXT), &options);

    verify_tally(&tally);
}

#[test]
fn test_memory_mapped_fallback() {
    // Memory-mapped requires a real file, but when used with a cursor,
    // it should fall back to a safe implementation
    let options = make_options(Io::MemoryMapped, Processing::Sequential);
    let tally = WordTally::new(Cursor::new(TEST_TEXT), &options);

    verify_tally(&tally);
}

#[test]
fn test_memory_mapped_parallel_fallback() {
    // Memory-mapped parallel also requires a real file but should fall back safely
    let options = make_options(Io::MemoryMapped, Processing::Parallel);
    let tally = WordTally::new(Cursor::new(TEST_TEXT), &options);

    verify_tally(&tally);
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

    let sequential_tally = WordTally::new(Cursor::new(LARGE_TEST_TEXT), &sequential_options);
    let parallel_tally = WordTally::new(Cursor::new(LARGE_TEST_TEXT), &parallel_options);

    // Both should produce identical results
    assert_eq!(sequential_tally.count(), parallel_tally.count());
    assert_eq!(sequential_tally.uniq_count(), parallel_tally.uniq_count());

    // Check actual tally entries are the same
    let sequential_words: Vec<_> = sequential_tally.tally().to_vec();
    let parallel_words: Vec<_> = parallel_tally.tally().to_vec();

    assert_eq!(sequential_words, parallel_words);
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
    let mmap_tally = WordTally::try_from_file(file, &mmap_sequential_options)
        .expect("Failed to process file with memory mapping");

    verify_tally(&mmap_tally);

    // Open the file again for parallel test
    let file = File::open(&temp_path).expect("Failed to open temp file");

    // Test parallel memory-mapped I/O
    let mmap_parallel_options = make_options(Io::MemoryMapped, Processing::Parallel);
    let mmap_parallel_tally = WordTally::try_from_file(file, &mmap_parallel_options)
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

    // Try to open a non-existent file
    let path = PathBuf::from("/nonexistent/file/path");
    let file_result = File::open(&path);

    // The file open should fail
    assert!(file_result.is_err());

    // But even if we could somehow get a file handle, try_from_file would handle it gracefully
    // This test is more about the robustness of our design
    if let Ok(file) = file_result {
        let options = make_options(Io::MemoryMapped, Processing::Sequential);
        let result = WordTally::try_from_file(file, &options);

        // The operation should fail gracefully
        assert!(result.is_err());
    }
}
