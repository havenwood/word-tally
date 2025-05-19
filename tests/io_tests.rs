use std::io::Write;
use word_tally::{Count, Input, Io, Options, Performance, Processing, WordTally};

const TEST_TEXT: &str = "The quick brown fox
jumps over the lazy dog
Pack my box with five dozen liquor jugs";

const EXPECTED_WORD_COUNT: Count = 16;
const EXPECTED_TOTAL_COUNT: Count = 17;

fn make_options(io: Io, processing: Processing) -> Options {
    Options::default().with_io(io).with_processing(processing)
}

fn verify_tally(tally: &WordTally<'_>) {
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

    let mut temp_file = tempfile::NamedTempFile::new().expect("create temp file");
    Write::write_all(&mut temp_file, TEST_TEXT.as_bytes()).expect("process test");

    let input = Input::new(
        temp_file.path().to_str().expect("temp file path"),
        options.io(),
    )
    .expect("Failed to create Input");

    let tally = WordTally::new(&input, &options).expect("Failed to create WordTally");

    verify_tally(&tally);
}

#[test]
fn test_streamed_parallel() {
    let options = make_options(Io::Streamed, Processing::Parallel);

    let mut temp_file = tempfile::NamedTempFile::new().expect("create temp file");
    Write::write_all(&mut temp_file, TEST_TEXT.as_bytes()).expect("process test");

    let input = Input::new(
        temp_file.path().to_str().expect("temp file path"),
        options.io(),
    )
    .expect("Failed to create Input");

    let tally = WordTally::new(&input, &options).expect("Failed to create WordTally");

    verify_tally(&tally);
}

#[test]
fn test_buffered_sequential() {
    let options = make_options(Io::Buffered, Processing::Sequential);

    let mut temp_file = tempfile::NamedTempFile::new().expect("create temp file");
    Write::write_all(&mut temp_file, TEST_TEXT.as_bytes()).expect("process test");

    let input = Input::new(
        temp_file.path().to_str().expect("temp file path"),
        options.io(),
    )
    .expect("Failed to create Input");

    let tally = WordTally::new(&input, &options).expect("Failed to create WordTally");

    verify_tally(&tally);
}

#[test]
fn test_buffered_parallel() {
    let options = make_options(Io::Buffered, Processing::Parallel);

    let mut temp_file = tempfile::NamedTempFile::new().expect("create temp file");
    Write::write_all(&mut temp_file, TEST_TEXT.as_bytes()).expect("process test");

    let input = Input::new(
        temp_file.path().to_str().expect("temp file path"),
        options.io(),
    )
    .expect("Failed to create Input");

    let tally = WordTally::new(&input, &options).expect("Failed to create WordTally");

    verify_tally(&tally);
}

#[test]
fn test_new_with_io_combinations() {
    let mut temp_file = tempfile::NamedTempFile::new().expect("create temp file");
    Write::write_all(&mut temp_file, TEST_TEXT.as_bytes()).expect("process test");
    let file_path = temp_file.path().to_str().expect("temp file path");

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

    assert_eq!(sequential_tally.count(), parallel_tally.count());
    assert_eq!(sequential_tally.uniq_count(), parallel_tally.uniq_count());

    let sequential_words: Vec<_> = sequential_tally.tally().to_vec();
    let parallel_words: Vec<_> = parallel_tally.tally().to_vec();

    let seq_map: std::collections::HashMap<_, _> = sequential_words.into_iter().collect();
    let par_map: std::collections::HashMap<_, _> = parallel_words.into_iter().collect();

    assert_eq!(seq_map, par_map);
}

#[test]
fn test_memory_mapped_with_real_file() {
    let mut temp_file = tempfile::NamedTempFile::new().expect("create temp file");
    Write::write_all(&mut temp_file, TEST_TEXT.as_bytes()).expect("process test");
    let file_path = temp_file.path().to_str().expect("temp file path");

    let mmap_sequential_options = make_options(Io::MemoryMapped, Processing::Sequential);
    let sequential_input = Input::new(file_path, mmap_sequential_options.io())
        .expect("Failed to create sequential memory-mapped input");

    let mmap_tally = WordTally::new(&sequential_input, &mmap_sequential_options)
        .expect("Failed to process file with memory mapping");

    verify_tally(&mmap_tally);

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

    let temp_dir = tempfile::tempdir().expect("process test");
    let file_path = temp_dir.path().join("test_io.txt");
    std::fs::write(&file_path, TEST_TEXT).expect("process test");

    let file_input = Input::new(&file_path, Io::Streamed).expect("create test input");
    let mmap_input = Input::new(&file_path, Io::MemoryMapped).expect("create test input");
    let bytes_input = Input::from_bytes(TEST_TEXT);

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
    let mut temp_file = tempfile::NamedTempFile::new().expect("create temp file");
    Write::write_all(&mut temp_file, TEST_TEXT.as_bytes()).expect("process test");
    let file_path = temp_file.path().to_str().expect("temp file path");

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

    let mut temp_file = tempfile::NamedTempFile::new().expect("create temp file");
    Write::write_all(&mut temp_file, test_text.as_bytes()).expect("process test");
    let file_path = temp_file.path().to_str().expect("temp file path");

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

//
// Unit tests for Io enum traits and methods
//

#[test]
fn test_io_default() {
    assert_eq!(Io::default(), Io::Streamed);
}

#[test]
fn test_io_display_all_variants() {
    assert_eq!(Io::Streamed.to_string(), "streamed");
    assert_eq!(Io::Buffered.to_string(), "buffered");
    assert_eq!(Io::MemoryMapped.to_string(), "memory-mapped");
    assert_eq!(Io::Bytes.to_string(), "bytes");
}

#[test]
fn test_io_traits_partial_eq() {
    assert_eq!(Io::Streamed, Io::Streamed);
    assert_ne!(Io::Streamed, Io::Buffered);
    assert_ne!(Io::Buffered, Io::MemoryMapped);
    assert_ne!(Io::MemoryMapped, Io::Bytes);
}

#[test]
fn test_io_traits_ordering() {
    use std::cmp::Ordering;

    // Test PartialOrd and Ord
    assert!(Io::MemoryMapped < Io::Streamed);
    assert!(Io::Streamed < Io::Buffered);
    assert!(Io::Buffered < Io::Bytes);

    assert_eq!(Io::Streamed.cmp(&Io::Streamed), Ordering::Equal);
    assert_eq!(Io::MemoryMapped.cmp(&Io::Streamed), Ordering::Less);
    assert_eq!(Io::Bytes.cmp(&Io::Buffered), Ordering::Greater);
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
    assert_eq!(calculate_hash(&Io::Streamed), calculate_hash(&Io::Streamed));

    // Different values should (likely) have different hashes
    assert_ne!(calculate_hash(&Io::Streamed), calculate_hash(&Io::Buffered));
}

#[test]
fn test_io_traits_clone_copy() {
    let io1 = Io::MemoryMapped;
    let io2 = io1; // Copy trait
    let io3 = io1; // Clone trait

    assert_eq!(io1, io2);
    assert_eq!(io1, io3);
    assert_eq!(io2, io3);
}

#[test]
fn test_io_serialization() {
    // Test Serialize
    let io = Io::MemoryMapped;
    let serialized = serde_json::to_string(&io).expect("serialize JSON");
    assert_eq!(serialized, "\"MemoryMapped\"");

    // Test Deserialize
    let deserialized: Io = serde_json::from_str("\"Buffered\"").expect("deserialize JSON");
    assert_eq!(deserialized, Io::Buffered);

    // Test roundtrip
    let original = Io::Bytes;
    let json = serde_json::to_string(&original).expect("serialize JSON");
    let roundtrip: Io = serde_json::from_str(&json).expect("deserialize JSON");
    assert_eq!(original, roundtrip);
}

#[test]
fn test_parse_io_from_env() {
    use word_tally::options::io::{ENV_IO, parse_io_from_env};

    // Test with no environment variable (should return default)
    unsafe {
        std::env::remove_var(ENV_IO);
    }
    assert_eq!(parse_io_from_env(), Io::default());

    // Test case-insensitive parsing
    unsafe {
        std::env::set_var(ENV_IO, "STREAMED");
    }
    assert_eq!(parse_io_from_env(), Io::Streamed);

    unsafe {
        std::env::set_var(ENV_IO, "buffered");
    }
    assert_eq!(parse_io_from_env(), Io::Buffered);

    unsafe {
        std::env::set_var(ENV_IO, "Memory-Mapped");
    }
    assert_eq!(parse_io_from_env(), Io::MemoryMapped);

    // Test mmap alias
    unsafe {
        std::env::set_var(ENV_IO, "mmap");
    }
    assert_eq!(parse_io_from_env(), Io::MemoryMapped);

    unsafe {
        std::env::set_var(ENV_IO, "MMAP");
    }
    assert_eq!(parse_io_from_env(), Io::MemoryMapped);

    // Test invalid values (should return default)
    unsafe {
        std::env::set_var(ENV_IO, "invalid");
    }
    assert_eq!(parse_io_from_env(), Io::default());

    unsafe {
        std::env::set_var(ENV_IO, "");
    }
    assert_eq!(parse_io_from_env(), Io::default());

    // Clean up
    unsafe {
        std::env::remove_var(ENV_IO);
    }
}

#[test]
fn test_env_io_constant() {
    use word_tally::options::io::ENV_IO;

    assert_eq!(ENV_IO, "WORD_TALLY_IO");
}

#[test]
fn test_io_exhaustive_matching() {
    // This test ensures all variants are covered
    let test_io = |io: Io| match io {
        Io::MemoryMapped => "mmap",
        Io::Streamed => "stream",
        Io::Buffered => "buffer",
        Io::Bytes => "bytes",
    };

    assert_eq!(test_io(Io::MemoryMapped), "mmap");
    assert_eq!(test_io(Io::Streamed), "stream");
    assert_eq!(test_io(Io::Buffered), "buffer");
    assert_eq!(test_io(Io::Bytes), "bytes");
}

#[test]
fn test_io_debug_format() {
    // Test Debug trait
    assert_eq!(format!("{:?}", Io::Streamed), "Streamed");
    assert_eq!(format!("{:?}", Io::Buffered), "Buffered");
    assert_eq!(format!("{:?}", Io::MemoryMapped), "MemoryMapped");
    assert_eq!(format!("{:?}", Io::Bytes), "Bytes");
}
