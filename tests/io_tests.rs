//! Tests for I/O functionality.

use hashbrown::HashMap;
use std::fs;
use std::io::Write;

use anyhow::Context;
use word_tally::{Count, Input, Io, Options, Performance, WordTally};

const TEST_TEXT: &str = "The quick brown fox
jumps over the lazy dog
Pack my box with five dozen liquor jugs";

const EXPECTED_WORD_COUNT: Count = 17;
const EXPECTED_TOTAL_COUNT: Count = 17;

fn make_options(io: Io) -> Options {
    Options::default().with_io(io)
}

fn verify_tally(tally: &WordTally) {
    assert_eq!(
        tally.count(),
        EXPECTED_TOTAL_COUNT,
        "Total word count mismatch. Expected {} words, found {}",
        EXPECTED_TOTAL_COUNT,
        tally.count()
    );
    assert_eq!(
        tally.uniq_count(),
        EXPECTED_WORD_COUNT,
        "Unique word count mismatch"
    );
}

#[test]
fn test_parallel_stream_sequential() {
    let options = make_options(Io::ParallelStream);

    let mut temp_file = tempfile::NamedTempFile::new().expect("create temp file");
    Write::write_all(&mut temp_file, TEST_TEXT.as_bytes()).expect("process test");

    let input = Input::new(
        temp_file.path().to_str().expect("temp file path"),
        options.io(),
    )
    .expect("create input");

    let tally = WordTally::new(&input, &options).expect("create word tally");

    verify_tally(&tally);
}

#[test]
fn test_parallel_stream_parallel() {
    let options = make_options(Io::ParallelStream);

    let mut temp_file = tempfile::NamedTempFile::new().expect("create temp file");
    Write::write_all(&mut temp_file, TEST_TEXT.as_bytes()).expect("process test");

    let input = Input::new(
        temp_file.path().to_str().expect("temp file path"),
        options.io(),
    )
    .expect("create input");

    let tally = WordTally::new(&input, &options).expect("create word tally");

    verify_tally(&tally);
}

#[test]
fn test_parallel_in_memory_sequential() {
    let options = make_options(Io::ParallelInMemory);

    let mut temp_file = tempfile::NamedTempFile::new().expect("create temp file");
    Write::write_all(&mut temp_file, TEST_TEXT.as_bytes()).expect("process test");

    let input = Input::new(
        temp_file.path().to_str().expect("temp file path"),
        options.io(),
    )
    .expect("create input");

    let tally = WordTally::new(&input, &options).expect("create word tally");

    verify_tally(&tally);
}

#[test]
fn test_parallel_in_memory_parallel() {
    let options = make_options(Io::ParallelInMemory);

    let mut temp_file = tempfile::NamedTempFile::new().expect("create temp file");
    Write::write_all(&mut temp_file, TEST_TEXT.as_bytes()).expect("process test");

    let input = Input::new(
        temp_file.path().to_str().expect("temp file path"),
        options.io(),
    )
    .expect("create input");

    let tally = WordTally::new(&input, &options).expect("create word tally");

    verify_tally(&tally);
}

#[test]
fn test_new_with_io_combinations() -> anyhow::Result<()> {
    let mut temp_file = tempfile::NamedTempFile::new()?;
    Write::write_all(&mut temp_file, TEST_TEXT.as_bytes())?;
    let file_path = temp_file
        .path()
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("invalid temp file path"))?;

    let io_strategies = [Io::ParallelStream, Io::ParallelInMemory, Io::ParallelMmap];

    for &io in &io_strategies {
        let options = make_options(io);
        let input = Input::new(file_path, io)
            .with_context(|| format!("input creation failed with `{io:?}`"))?;

        let tally = WordTally::new(&input, &options)
            .with_context(|| format!("`new()` failed with `{io:?}`"))?;

        verify_tally(&tally);
    }

    Ok(())
}

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

#[test]
fn test_parallel_processing_with_large_text() {
    let mut temp_file = tempfile::NamedTempFile::new().expect("create temp file");
    Write::write_all(&mut temp_file, LARGE_TEST_TEXT.as_bytes()).expect("process test");
    let file_path = temp_file.path().to_str().expect("temp file path");

    let sequential_options = make_options(Io::ParallelInMemory);
    let parallel_options = make_options(Io::ParallelInMemory);

    let sequential_input =
        Input::new(file_path, sequential_options.io()).expect("create sequential input");
    let parallel_input =
        Input::new(file_path, parallel_options.io()).expect("create parallel input");

    let sequential_tally = WordTally::new(&sequential_input, &sequential_options)
        .expect("create sequential WordTally");
    let parallel_tally =
        WordTally::new(&parallel_input, &parallel_options).expect("create parallel WordTally");

    assert_eq!(sequential_tally.count(), parallel_tally.count());
    assert_eq!(sequential_tally.uniq_count(), parallel_tally.uniq_count());

    let sequential_words: Vec<_> = sequential_tally.tally().to_vec();
    let parallel_words: Vec<_> = parallel_tally.tally().to_vec();

    let seq_map: HashMap<_, _> = sequential_words.into_iter().collect();
    let par_map: HashMap<_, _> = parallel_words.into_iter().collect();

    assert_eq!(seq_map, par_map);
}

#[test]
fn test_parallel_mmap_with_real_file() {
    let mut temp_file = tempfile::NamedTempFile::new().expect("create temp file");
    Write::write_all(&mut temp_file, TEST_TEXT.as_bytes()).expect("process test");
    let file_path = temp_file.path().to_str().expect("temp file path");

    let mmap_options = make_options(Io::ParallelMmap);
    let input = Input::new(file_path, mmap_options.io()).expect("create memory-mapped input");

    let mmap_tally =
        WordTally::new(&input, &mmap_options).expect("process file with memory mapping");

    verify_tally(&mmap_tally);
}

#[test]
fn test_read_trait_with_all_io_strategies() {
    use std::io::Read;

    let temp_dir = tempfile::tempdir().expect("process test");
    let file_path = temp_dir.path().join("test_io.txt");
    fs::write(&file_path, TEST_TEXT).expect("process test");

    let file_input = Input::new(&file_path, Io::ParallelStream).expect("create test input");
    let mmap_input = Input::new(&file_path, Io::ParallelMmap).expect("create test input");
    let bytes_input = Input::from_bytes(TEST_TEXT);

    let test_cases = [file_input, mmap_input, bytes_input];

    for input in &test_cases {
        let mut reader = input.reader().expect("create reader");
        let mut content = String::new();
        reader.read_to_string(&mut content).expect("read content");

        assert_eq!(content.trim(), TEST_TEXT.trim());
    }
}

#[test]
fn test_bytes_input() {
    let options = make_options(Io::ParallelBytes);
    let input = Input::from_bytes(TEST_TEXT);

    assert_eq!(input.source(), "<bytes>");
    assert_eq!(input.size(), Some(TEST_TEXT.len() as u64));

    let tally = WordTally::new(&input, &options).expect("create word tally");
    verify_tally(&tally);
}

#[test]
fn test_bytes_io_with_input_new() {
    let result = Input::new(TEST_TEXT, Io::ParallelBytes);
    assert!(result.is_err());
    let err = result
        .expect_err("bytes I/O mode with Input::new should fail")
        .to_string();
    assert!(err.contains("byte I/O mode requires `Input::from_bytes()`"));
}

#[test]
fn test_nonexistent_file_handling() {
    use std::fs::File;
    use std::path::PathBuf;

    let path = PathBuf::from("/nonexistent/file/path");
    let file_result = File::open(&path);
    assert!(file_result.is_err());
}

#[test]
fn test_new_with_all_io_strategies() -> anyhow::Result<()> {
    let mut temp_file = tempfile::NamedTempFile::new()?;
    Write::write_all(&mut temp_file, TEST_TEXT.as_bytes())?;
    let file_path = temp_file
        .path()
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("invalid temp file path"))?;

    let io_strategies = [Io::ParallelStream, Io::ParallelInMemory, Io::ParallelMmap];

    for &io in &io_strategies {
        let options = make_options(io);
        let input = Input::new(file_path, io)
            .with_context(|| format!("input creation failed with `{io:?}`"))?;

        let tally = WordTally::new(&input, &options)
            .with_context(|| format!("`new()` failed with `{io:?}`"))?;

        verify_tally(&tally);
    }

    Ok(())
}

#[test]
fn test_utf8_boundary_handling() {
    let test_text = "æ æ æ æ";

    let mut temp_file = tempfile::NamedTempFile::new().expect("create temp file");
    Write::write_all(&mut temp_file, test_text.as_bytes()).expect("process test");
    let file_path = temp_file.path().to_str().expect("temp file path");

    let small_chunk_size = 32;
    let performance = Performance::default().with_chunk_size(small_chunk_size);

    let options = Options::default()
        .with_io(Io::ParallelMmap)
        .with_performance(performance);

    let input = Input::new(file_path, options.io()).expect("create input");
    let tally = WordTally::new(&input, &options).expect("create word tally");

    assert!(
        tally
            .tally()
            .iter()
            .map(|(word, _)| word)
            .any(|word| word == &"æ".into()),
        "Missing 'æ' in results"
    );
}

//
// Unit tests for Io enum traits and methods
//

#[test]
fn test_io_default() {
    assert_eq!(Io::default(), Io::ParallelStream);
}

#[test]
fn test_io_display_all_variants() {
    assert_eq!(Io::ParallelStream.to_string(), "parallel-stream");
    assert_eq!(Io::ParallelInMemory.to_string(), "parallel-in-memory");
    assert_eq!(Io::ParallelMmap.to_string(), "parallel-mmap");
    assert_eq!(Io::ParallelBytes.to_string(), "parallel-bytes");
}

#[test]
fn test_io_traits_partial_eq() {
    assert_eq!(Io::ParallelStream, Io::ParallelStream);
    assert_ne!(Io::ParallelStream, Io::ParallelInMemory);
    assert_ne!(Io::ParallelInMemory, Io::ParallelMmap);
    assert_ne!(Io::ParallelMmap, Io::ParallelBytes);
}

#[test]
fn test_io_traits_ordering() {
    use std::cmp::Ordering;

    // Test PartialOrd and Ord (based on enum declaration order)
    assert!(Io::ParallelStream < Io::Stream);
    assert!(Io::Stream < Io::ParallelMmap);
    assert!(Io::ParallelMmap < Io::ParallelBytes);
    assert!(Io::ParallelBytes < Io::ParallelInMemory);

    assert_eq!(Io::ParallelStream.cmp(&Io::ParallelStream), Ordering::Equal);
    assert_eq!(Io::Stream.cmp(&Io::ParallelStream), Ordering::Greater);
    assert_eq!(Io::ParallelBytes.cmp(&Io::ParallelInMemory), Ordering::Less);
}

#[test]
fn test_io_traits_hash() {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    fn calculate_hash<T: Hash>(t: &T) -> u64 {
        let mut s = DefaultHasher::new();
        t.hash(&mut s);
        s.finish()
    }

    // Same values should have same hash
    assert_eq!(
        calculate_hash(&Io::ParallelStream),
        calculate_hash(&Io::ParallelStream)
    );

    // Different values should (likely) have different hashes
    assert_ne!(
        calculate_hash(&Io::ParallelStream),
        calculate_hash(&Io::ParallelInMemory)
    );
}

#[test]
fn test_io_traits_clone_copy() {
    let io1 = Io::ParallelMmap;
    let io2 = io1; // Copy trait
    let io3 = io1; // Clone trait

    assert_eq!(io1, io2);
    assert_eq!(io1, io3);
    assert_eq!(io2, io3);
}

#[test]
fn test_io_serialization() {
    // Test Serialize
    let io = Io::ParallelMmap;
    let serialized = serde_json::to_string(&io).expect("serialize JSON");
    assert_eq!(serialized, "\"parallelMmap\"");

    // Test Deserialize
    let deserialized: Io = serde_json::from_str("\"parallelInMemory\"").expect("deserialize JSON");
    assert_eq!(deserialized, Io::ParallelInMemory);

    // Test roundtrip
    let original = Io::ParallelBytes;
    let json = serde_json::to_string(&original).expect("serialize JSON");
    let roundtrip: Io = serde_json::from_str(&json).expect("deserialize JSON");
    assert_eq!(original, roundtrip);
}

#[test]
fn test_parse_io_from_str_value() {
    // Test with no value (should return default)
    assert_eq!(Io::from_str_value(None), Io::default());

    // Test case-insensitive parsing
    assert_eq!(Io::from_str_value(Some("stream")), Io::Stream);
    assert_eq!(
        Io::from_str_value(Some("PARALLEL-STREAM")),
        Io::ParallelStream
    );
    assert_eq!(
        Io::from_str_value(Some("parallel-in-memory")),
        Io::ParallelInMemory
    );
    assert_eq!(Io::from_str_value(Some("Parallel-Mmap")), Io::ParallelMmap);

    // Test mmap alias
    assert_eq!(Io::from_str_value(Some("mmap")), Io::ParallelMmap);
    assert_eq!(Io::from_str_value(Some("MMAP")), Io::ParallelMmap);

    // Test invalid values (should return default)
    assert_eq!(Io::from_str_value(Some("invalid")), Io::default());
    assert_eq!(Io::from_str_value(Some("")), Io::default());
}

#[test]
fn test_io_from_str_trait() {
    use std::str::FromStr;

    // Test valid values
    assert_eq!(Io::from_str("stream"), Ok(Io::Stream));
    assert_eq!(Io::from_str("STREAM"), Ok(Io::Stream));
    assert_eq!(Io::from_str("parallel-stream"), Ok(Io::ParallelStream));
    assert_eq!(Io::from_str("PARALLEL-STREAM"), Ok(Io::ParallelStream));
    assert_eq!(Io::from_str("parallel-in-memory"), Ok(Io::ParallelInMemory));
    assert_eq!(Io::from_str("Parallel-In-Memory"), Ok(Io::ParallelInMemory));
    assert_eq!(Io::from_str("parallel-mmap"), Ok(Io::ParallelMmap));
    assert_eq!(Io::from_str("PARALLEL-MMAP"), Ok(Io::ParallelMmap));
    assert_eq!(Io::from_str("mmap"), Ok(Io::ParallelMmap));
    assert_eq!(Io::from_str("MMAP"), Ok(Io::ParallelMmap));
    assert_eq!(Io::from_str("parallel-bytes"), Ok(Io::ParallelBytes));
    assert_eq!(Io::from_str("PARALLEL-BYTES"), Ok(Io::ParallelBytes));

    // Test invalid values
    assert!(Io::from_str("invalid").is_err());
    assert!(Io::from_str("").is_err());
    assert!(Io::from_str("stream-parallel").is_err());

    // Test parse() method works too
    assert_eq!("stream".parse::<Io>(), Ok(Io::Stream));
    assert_eq!("parallel-mmap".parse::<Io>(), Ok(Io::ParallelMmap));
    assert!("invalid".parse::<Io>().is_err());
}

#[test]
fn test_env_io_constant() {
    assert_eq!(Io::ENV_IO, "WORD_TALLY_IO");
}

#[test]
fn test_io_exhaustive_matching() {
    // This test ensures all variants are covered
    let test_io = |io: Io| match io {
        Io::Stream => "low-memory",
        Io::ParallelMmap => "mmap",
        Io::ParallelStream => "stream",
        Io::ParallelInMemory => "in-memory",
        Io::ParallelBytes => "parallel-bytes",
    };

    assert_eq!(test_io(Io::Stream), "low-memory");
    assert_eq!(test_io(Io::ParallelMmap), "mmap");
    assert_eq!(test_io(Io::ParallelStream), "stream");
    assert_eq!(test_io(Io::ParallelInMemory), "in-memory");
    assert_eq!(test_io(Io::ParallelBytes), "parallel-bytes");
}

#[test]
fn test_io_debug_format() {
    // Test Debug trait
    assert_eq!(format!("{:?}", Io::ParallelStream), "ParallelStream");
    assert_eq!(format!("{:?}", Io::ParallelInMemory), "ParallelInMemory");
    assert_eq!(format!("{:?}", Io::ParallelMmap), "ParallelMmap");
    assert_eq!(format!("{:?}", Io::ParallelBytes), "ParallelBytes");
}

//
// Tests for streaming chunk size handling
//

use std::fs::File;
use std::sync::Arc;
use tempfile::TempDir;
use word_tally::{Case, Filters, Serialization, Sort};

fn make_shared<T>(value: T) -> Arc<T> {
    Arc::new(value)
}

/// Generate a simple test file with known content.
fn create_test_file_with_size(dir: &TempDir, size_mb: usize) -> anyhow::Result<String> {
    let path = dir.path().join(format!("test_{size_mb}mb.txt"));
    let mut file = File::create(&path)?;

    // Simple pattern: 10 "narrow" + 10 "certain" per line
    let pattern = "narrow narrow narrow narrow narrow narrow narrow narrow narrow narrow certain certain certain certain certain certain certain certain certain certain\n";
    let pattern_bytes = pattern.as_bytes();
    let target_bytes = size_mb * 1024 * 1024;
    let mut written = 0;

    while written < target_bytes {
        file.write_all(pattern_bytes)?;
        written += pattern_bytes.len();
    }

    Ok(path.to_string_lossy().to_string())
}

#[test]
fn test_streaming_processes_entire_file() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;

    // Create a 5MB file - larger than typical batch size
    let file_path = create_test_file_with_size(&temp_dir, 5)?;

    let base_options = Options::new(
        Case::default(),
        Sort::default(),
        Serialization::default(),
        Filters::default(),
        Io::ParallelStream,
        Performance::default(),
    );

    // Test streaming
    let streaming_options = make_shared(base_options.clone().with_io(Io::ParallelStream));
    let streaming_input = Input::new(&file_path, streaming_options.io())?;
    let streaming_tally = WordTally::new(&streaming_input, &streaming_options)?;
    let streaming_count: usize = streaming_tally
        .into_iter()
        .find(|(w, _)| &**w == "narrow")
        .map_or(0, |(_, c)| c);

    // Test in-memory for comparison
    let in_memory_options = make_shared(base_options.with_io(Io::ParallelInMemory));
    let in_memory_input = Input::new(&file_path, in_memory_options.io())?;
    let in_memory_tally = WordTally::new(&in_memory_input, &in_memory_options)?;
    let in_memory_count: usize = in_memory_tally
        .into_iter()
        .find(|(w, _)| &**w == "narrow")
        .map_or(0, |(_, c)| c);

    assert_eq!(
        streaming_count, in_memory_count,
        "Streaming count ({streaming_count}) differs from in-memory count ({in_memory_count})"
    );

    Ok(())
}

#[test]
fn test_streaming_consistency_across_io_modes() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;
    let file_path = create_test_file_with_size(&temp_dir, 2)?; // 2MB file

    // Test with streaming
    let performance = Performance::default();
    let filters = Filters::default();
    let streaming_options = Options::new(
        Case::default(),
        Sort::default(),
        Serialization::default(),
        filters.clone(),
        Io::ParallelStream,
        performance,
    );
    let streaming_options_arc = make_shared(streaming_options);

    let streaming_input = Input::new(&file_path, streaming_options_arc.io())?;
    let streaming_tally = WordTally::new(&streaming_input, &streaming_options_arc)?;
    let streaming_results: Vec<_> = streaming_tally.into_iter().collect();

    // Test with in-memory
    let in_memory_options = Options::new(
        Case::default(),
        Sort::default(),
        Serialization::default(),
        filters,
        Io::ParallelInMemory,
        performance,
    );
    let in_memory_options_arc = make_shared(in_memory_options);

    let in_memory_input = Input::new(&file_path, in_memory_options_arc.io())?;
    let in_memory_tally = WordTally::new(&in_memory_input, &in_memory_options_arc)?;
    let in_memory_results: Vec<_> = in_memory_tally.into_iter().collect();

    assert_eq!(
        streaming_results.len(),
        in_memory_results.len(),
        "Different number of unique words"
    );

    // Convert to HashMap for order-independent comparison
    let streaming_map: HashMap<_, _> = streaming_results.into_iter().collect();
    let in_memory_map: HashMap<_, _> = in_memory_results.into_iter().collect();

    assert_eq!(
        streaming_map, in_memory_map,
        "Results differ between streaming and in-memory modes"
    );

    Ok(())
}
