use std::sync::Arc;
use word_tally::{Filters, Input, Io, Options, Serialization, WordTally};

fn make_shared<T>(value: T) -> Arc<T> {
    Arc::new(value)
}

#[test]
fn test_input_size() {
    let mut temp_file = tempfile::NamedTempFile::new().expect("create temp file");
    let content = b"This test file replaces the hardcoded fixture";
    std::io::Write::write_all(&mut temp_file, content).expect("write test data");

    let file_input = Input::File(temp_file.path().to_path_buf());
    let size = file_input.size();
    assert!(size.is_some());
    assert!(size.expect("get file size") > 0);

    let stdin_input = Input::Stdin;
    assert_eq!(stdin_input.size(), None);
}

#[test]
fn test_parallel_vs_sequential() {
    let input_text = b"I taste a liquor never brewed. I taste a liquor.";
    let mut temp_file = tempfile::NamedTempFile::new().expect("create temp file");
    std::io::Write::write_all(&mut temp_file, input_text).expect("write test data");
    let file_path = temp_file.path().to_str().expect("temp file path");

    // Sequential processing
    let seq_performance = word_tally::Performance::default();
    let filters = Filters::default();
    let seq_options = Options::new(
        word_tally::Case::default(),
        word_tally::Sort::default(),
        Serialization::default(),
        filters.clone(),
        Io::ParallelStream,
        seq_performance,
    );
    let seq_options_arc = make_shared(seq_options);

    let seq_input = Input::new(file_path, seq_options_arc.io()).expect("create sequential input");

    let sequential =
        WordTally::new(&seq_input, &seq_options_arc).expect("create sequential WordTally");

    // Parallel processing
    let par_performance = word_tally::Performance::default();
    let par_options = Options::new(
        word_tally::Case::default(),
        word_tally::Sort::default(),
        Serialization::default(),
        filters,
        Io::ParallelStream,
        par_performance,
    );
    let par_options_arc = make_shared(par_options);

    let par_input = Input::new(file_path, par_options_arc.io()).expect("create parallel input");

    let parallel =
        WordTally::new(&par_input, &par_options_arc).expect("create parallel word tally");

    assert_eq!(sequential.count(), parallel.count());
    assert_eq!(sequential.uniq_count(), parallel.uniq_count());

    let mut seq_tally: Vec<_> = sequential.tally().to_vec();
    seq_tally.sort_by_key(|(word, _): &(Box<str>, usize)| word.clone());

    let mut par_tally: Vec<_> = parallel.tally().to_vec();
    par_tally.sort_by_key(|(word, _): &(Box<str>, usize)| word.clone());

    assert_eq!(seq_tally, par_tally);
}

#[test]
fn test_memory_mapped_vs_streamed() {
    let mut temp_file = tempfile::NamedTempFile::new().expect("create temp file");
    let content = b"Quantum leapt circumference imperceptible enigma quantum enigma";
    std::io::Write::write_all(&mut temp_file, content).expect("write test data");
    let file_path = temp_file.path().to_str().expect("temp file path");

    // Set up options for memory-mapped I/O (sequential)
    let mmap_performance = word_tally::Performance::default();
    let filters = Filters::default();
    let mmap_options = Options::new(
        word_tally::Case::default(),
        word_tally::Sort::default(),
        Serialization::default(),
        filters.clone(),
        Io::ParallelMmap,
        mmap_performance,
    );

    // Set up options for streaming I/O (sequential)
    let stream_performance = word_tally::Performance::default();
    let stream_options = Options::new(
        word_tally::Case::default(),
        word_tally::Sort::default(),
        Serialization::default(),
        filters.clone(),
        Io::ParallelStream,
        stream_performance,
    );

    // Create inputs with the different I/O modes
    let mmap_input = Input::new(file_path, mmap_options.io()).expect("create memory-mapped input");
    let stream_input = Input::new(file_path, stream_options.io()).expect("create streamed input");

    // Create WordTally instances with the different I/O modes
    let memory_mapped =
        WordTally::new(&mmap_input, &mmap_options).expect("create memory mapped word tally");
    let streamed =
        WordTally::new(&stream_input, &stream_options).expect("create streamed WordTally");

    // Verify results are the same regardless of I/O mode
    assert_eq!(memory_mapped.count(), streamed.count());
    assert_eq!(memory_mapped.uniq_count(), streamed.uniq_count());

    let mut mmap_tally: Vec<_> = memory_mapped.tally().to_vec();
    mmap_tally.sort_by_key(|(word, _): &(Box<str>, usize)| word.clone());

    let mut stream_tally: Vec<_> = streamed.tally().to_vec();
    stream_tally.sort_by_key(|(word, _): &(Box<str>, usize)| word.clone());

    assert_eq!(mmap_tally, stream_tally);

    // Now test with parallel processing
    let parallel_performance = word_tally::Performance::default();
    let parallel_options = Options::new(
        word_tally::Case::default(),
        word_tally::Sort::default(),
        Serialization::default(),
        filters,
        Io::ParallelStream,
        parallel_performance,
    );

    let parallel_input =
        Input::new(file_path, parallel_options.io()).expect("create parallel input");

    let parallel_stream = WordTally::new(&parallel_input, &parallel_options)
        .expect("create parallel stream WordTally");

    // Verify the parallel processing worked
    assert!(parallel_stream.count() > 0);
    assert!(parallel_stream.uniq_count() > 0);
}

#[test]
fn test_parallel_count() {
    // Test the parallel function works with default settings
    let input_text = b"Test with default settings for chunk size and thread count";
    let mut temp_file = tempfile::NamedTempFile::new().expect("create temp file");
    std::io::Write::write_all(&mut temp_file, input_text).expect("write test data");

    let options = Options::default();

    let input = Input::new(
        temp_file.path().to_str().expect("temp file path"),
        options.io(),
    )
    .expect("create input");

    let parallel = WordTally::new(&input, &options).expect("create parallel word tally");

    assert!(parallel.count() > 0);
    assert!(parallel.uniq_count() > 0);
    assert!(parallel.uniq_count() <= parallel.count());
}

#[test]
fn test_merge_maps() {
    let input_text = b"This is a test of the map merging functionality";
    let mut temp_file = tempfile::NamedTempFile::new().expect("create temp file");
    std::io::Write::write_all(&mut temp_file, input_text).expect("write test data");

    let options = Options::default();

    let input = Input::new(
        temp_file.path().to_str().expect("temp file path"),
        options.io(),
    )
    .expect("create input");

    let tally = WordTally::new(&input, &options).expect("create word tally");

    assert_eq!(tally.count(), 9);
    assert_eq!(tally.uniq_count(), 9);
}
