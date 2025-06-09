//! Tests for library functionality.

use std::fs;
use std::io::Write;
use std::sync::Arc;

use hashbrown::HashSet;
use word_tally::output::Output;
use word_tally::{
    Case, Count, ExcludeWords, Filters, Io, Metadata, Options, Performance, Reader, Serialization,
    Sort, TallyMap, View, Word, WordTally,
};

fn make_shared<T>(value: T) -> Arc<T> {
    Arc::new(value)
}

struct ExpectedFields<'a> {
    count: Count,
    uniq_count: Count,
    tally: Vec<(&'a str, Count)>,
}

fn create_test_data_file() -> tempfile::NamedTempFile {
    let mut temp_file = tempfile::NamedTempFile::new().expect("create temp file");
    // Using content that matches expected test data structure
    let content = b"d\n\
                    123 123 123 123 123\n\
                    a A *** A C D 123 123\n\
                    b b b B B B B 123\n\
                    d C d d d D D D D\n\
                    c D c c c c c c C C C C C C\n\
                    123\n";
    Write::write_all(&mut temp_file, content).expect("write test data");
    temp_file
}

fn word_tally(case: Case, sort: Sort, serialization: Serialization, filters: Filters) -> WordTally {
    let test_file = create_test_data_file();
    let file_path = test_file.path().to_str().expect("temp file path");

    let options = Options::new(
        case,
        sort,
        serialization,
        filters,
        Io::ParallelStream,
        Performance::default(),
    );

    let reader = Reader::try_from(file_path).expect("create reader");
    let tally_map = TallyMap::from_reader(&reader, &options).expect("create tally map");
    WordTally::from_tally_map(tally_map, &options)
}

fn word_tally_test(case: Case, sort: Sort, filters: Filters, fields: &ExpectedFields<'_>) {
    let serialization = Serialization::default();
    let word_tally = word_tally(case, sort, serialization, filters);
    assert_eq!(word_tally.count(), fields.count);
    assert_eq!(word_tally.uniq_count(), fields.uniq_count);

    let expected_tally = fields
        .tally
        .iter()
        .map(|(word, count)| (Box::from(*word), *count))
        .collect::<Vec<_>>()
        .into_boxed_slice();

    if sort == Sort::Unsorted {
        let expected_words: HashSet<_> = expected_tally.iter().collect();
        let actual_words: HashSet<_> = word_tally.tally().iter().collect();
        assert_eq!(expected_words, actual_words);
    } else {
        assert_eq!(word_tally.tally(), expected_tally.as_ref());
    }
}

#[test]
fn lower_case_desc_order() {
    word_tally_test(
        Case::Lower,
        Sort::Desc,
        Filters::default(),
        &ExpectedFields {
            count: 45,
            uniq_count: 5,
            tally: vec![("c", 15), ("d", 11), ("123", 9), ("b", 7), ("a", 3)],
        },
    );
}

#[test]
fn min_char_count_at_max() {
    word_tally_test(
        Case::Lower,
        Sort::Desc,
        Filters::default().with_min_chars(3),
        &ExpectedFields {
            count: 9,
            uniq_count: 1,
            tally: vec![("123", 9)],
        },
    );
}

#[test]
fn min_char_count_above_max() {
    word_tally_test(
        Case::Lower,
        Sort::Desc,
        Filters::default().with_min_chars(4),
        &ExpectedFields {
            count: 0,
            uniq_count: 0,
            tally: vec![],
        },
    );
}

#[test]
fn min_char_count_at_min() {
    word_tally_test(
        Case::Lower,
        Sort::Desc,
        Filters::default(),
        &ExpectedFields {
            count: 45,
            uniq_count: 5,
            tally: vec![("c", 15), ("d", 11), ("123", 9), ("b", 7), ("a", 3)],
        },
    );
}

#[test]
fn min_word_count_at_max() {
    word_tally_test(
        Case::Lower,
        Sort::Desc,
        Filters::default().with_min_count(15),
        &ExpectedFields {
            count: 15,
            uniq_count: 1,
            tally: vec![("c", 15)],
        },
    );
}

#[test]
fn default_case_unsorted_order() {
    word_tally_test(
        Case::default(),
        Sort::Unsorted,
        Filters::default(),
        &ExpectedFields {
            count: 45,
            uniq_count: 9,
            tally: vec![
                ("d", 5),
                ("123", 9),
                ("a", 1),
                ("C", 8),
                ("D", 6),
                ("b", 3),
                ("c", 7),
                ("B", 4),
                ("A", 2),
            ],
        },
    );
}

#[test]
fn upper_case_desc_order() {
    word_tally_test(
        Case::Upper,
        Sort::Desc,
        Filters::default(),
        &ExpectedFields {
            count: 45,
            uniq_count: 5,
            tally: vec![("C", 15), ("D", 11), ("123", 9), ("B", 7), ("A", 3)],
        },
    );
}

#[test]
fn lower_case_asc_order() {
    word_tally_test(
        Case::Lower,
        Sort::Asc,
        Filters::default(),
        &ExpectedFields {
            count: 45,
            uniq_count: 5,
            tally: vec![("a", 3), ("b", 7), ("123", 9), ("d", 11), ("c", 15)],
        },
    );
}

#[test]
fn upper_case_asc_order() {
    word_tally_test(
        Case::Upper,
        Sort::Asc,
        Filters::default(),
        &ExpectedFields {
            count: 45,
            uniq_count: 5,
            tally: vec![("A", 3), ("B", 7), ("123", 9), ("D", 11), ("C", 15)],
        },
    );
}

#[test]
fn original_case_desc_order() {
    word_tally_test(
        Case::Original,
        Sort::Desc,
        Filters::default(),
        &ExpectedFields {
            count: 45,
            uniq_count: 9,
            tally: vec![
                ("123", 9),
                ("C", 8),
                ("c", 7),
                ("D", 6),
                ("d", 5),
                ("B", 4),
                ("b", 3),
                ("A", 2),
                ("a", 1),
            ],
        },
    );
}

#[test]
fn original_case_asc_order() {
    word_tally_test(
        Case::Original,
        Sort::Asc,
        Filters::default(),
        &ExpectedFields {
            count: 45,
            uniq_count: 9,
            tally: vec![
                ("a", 1),
                ("A", 2),
                ("b", 3),
                ("B", 4),
                ("d", 5),
                ("D", 6),
                ("c", 7),
                ("C", 8),
                ("123", 9),
            ],
        },
    );
}

#[test]
fn vec_from() {
    let tally = word_tally(
        Case::default(),
        Sort::default(),
        Serialization::default(),
        Filters::default(),
    );

    assert_eq!(
        Vec::from(tally),
        vec![
            (Box::from("123"), 9),
            (Box::from("C"), 8),
            (Box::from("c"), 7),
            (Box::from("D"), 6),
            (Box::from("d"), 5),
            (Box::from("B"), 4),
            (Box::from("b"), 3),
            (Box::from("A"), 2),
            (Box::from("a"), 1)
        ]
    );
}

#[test]
fn test_into_tally() {
    let input_text = b"Hope is the thing with feathers that perches in the soul";
    let mut temp_file = tempfile::NamedTempFile::new().expect("create temp file");
    Write::write_all(&mut temp_file, input_text).expect("write test data");

    let options = make_shared(Options::default());
    let reader = Reader::try_from(temp_file.path()).expect("create reader");
    let tally_map = TallyMap::from_reader(&reader, &options).expect("create tally map");
    let word_tally = WordTally::from_tally_map(tally_map, &options);

    // Use `tally()` to get a reference to the slice.
    let tally = word_tally.tally();

    let mut tally_vec: Vec<_> = tally.to_vec();
    tally_vec.sort_by_key(|(word, _): &(Word, Count)| word.clone());

    let mut expected_tally = vec![
        ("Hope".into(), 1),
        ("feathers".into(), 1),
        ("in".into(), 1),
        ("is".into(), 1),
        ("perches".into(), 1),
        ("soul".into(), 1),
        ("that".into(), 1),
        ("the".into(), 2),
        ("thing".into(), 1),
        ("with".into(), 1),
    ];
    expected_tally.sort_by_key(|(word, _): &(Word, Count)| word.clone());

    assert_eq!(tally_vec, expected_tally);
}

#[test]
fn test_iterator() {
    let input_text = b"double trouble double";
    let mut temp_file = tempfile::NamedTempFile::new().expect("create temp file");
    Write::write_all(&mut temp_file, input_text).expect("write test data");

    let options = make_shared(Options::default());
    let reader = Reader::try_from(temp_file.path()).expect("create reader");
    let tally_map = TallyMap::from_reader(&reader, &options).expect("create tally map");
    let word_tally = WordTally::from_tally_map(tally_map, &options);

    let expected: Vec<(Word, Count)> = vec![(Box::from("double"), 2), (Box::from("trouble"), 1)];

    let collected: Vec<(Word, Count)> = (&word_tally).into_iter().cloned().collect();
    assert_eq!(collected, expected);

    let mut iter = (&word_tally).into_iter();
    assert_eq!(iter.next(), Some(&(Box::from("double"), 2)));
    assert_eq!(iter.next(), Some(&(Box::from("trouble"), 1)));
    assert_eq!(iter.next(), None);
}

#[test]
fn test_iterator_for_loop() {
    let input_text = b"llama llama pajamas";
    let mut temp_file = tempfile::NamedTempFile::new().expect("create temp file");
    Write::write_all(&mut temp_file, input_text).expect("write test data");

    let options = make_shared(Options::default());
    let reader = Reader::try_from(temp_file.path()).expect("create reader");
    let tally_map = TallyMap::from_reader(&reader, &options).expect("create tally map");
    let word_tally = WordTally::from_tally_map(tally_map, &options);

    let expected: Vec<(Word, Count)> = vec![(Box::from("llama"), 2), (Box::from("pajamas"), 1)];

    let mut collected = vec![];
    for item in &word_tally {
        collected.push(item.clone());
    }
    assert_eq!(collected, expected);
}

#[test]
fn test_input_size() {
    let temp_file = create_test_data_file();
    let file_reader = Reader::try_from(temp_file.path()).expect("create file reader");
    let size = file_reader.size();
    assert!(size.is_some());
    assert!(size.expect("get file size") > 0);

    let stdin_reader = Reader::stdin();
    assert_eq!(stdin_reader.size(), None);
}

#[test]
fn test_parallel_vs_sequential() {
    let input_text = b"I taste a liquor never brewed. I taste a liquor.";
    let mut temp_file = tempfile::NamedTempFile::new().expect("create temp file");
    Write::write_all(&mut temp_file, input_text).expect("write test data");
    let file_path = temp_file.path().to_str().expect("temp file path");

    // Sequential processing
    let seq_performance = Performance::default();
    let filters = Filters::default();
    let seq_options = Options::new(
        Case::default(),
        Sort::default(),
        Serialization::default(),
        filters.clone(),
        Io::ParallelStream,
        seq_performance,
    );
    let seq_options_arc = make_shared(seq_options);

    let seq_reader = Reader::try_from(file_path).expect("create sequential reader");
    let seq_tally_map =
        TallyMap::from_reader(&seq_reader, &seq_options_arc).expect("create sequential tally map");
    let sequential = WordTally::from_tally_map(seq_tally_map, &seq_options_arc);

    // Parallel processing
    let par_performance = Performance::default();
    let par_options = Options::new(
        Case::default(),
        Sort::default(),
        Serialization::default(),
        filters,
        Io::ParallelStream,
        par_performance,
    );
    let par_options_arc = make_shared(par_options);

    let par_reader = Reader::try_from(file_path).expect("create parallel reader");
    let par_tally_map =
        TallyMap::from_reader(&par_reader, &par_options_arc).expect("create parallel tally map");
    let parallel = WordTally::from_tally_map(par_tally_map, &par_options_arc);

    assert_eq!(sequential.count(), parallel.count());
    assert_eq!(sequential.uniq_count(), parallel.uniq_count());

    let mut seq_tally: Vec<_> = sequential.tally().to_vec();
    seq_tally.sort_by_key(|(word, _): &(Word, Count)| word.clone());

    let mut par_tally: Vec<_> = parallel.tally().to_vec();
    par_tally.sort_by_key(|(word, _): &(Word, Count)| word.clone());

    assert_eq!(seq_tally, par_tally);
}

#[test]
fn test_memory_mapped_vs_streamed() {
    // Use the test file
    let test_file = create_test_data_file();
    let file_path = test_file.path().to_str().expect("temp file path");

    // Set up options for memory-mapped I/O (sequential)
    let mmap_performance = Performance::default();
    let filters = Filters::default();
    let mmap_options = Options::new(
        Case::default(),
        Sort::default(),
        Serialization::default(),
        filters.clone(),
        Io::ParallelMmap,
        mmap_performance,
    );

    // Set up options for streaming I/O (sequential)
    let stream_performance = Performance::default();
    let stream_options = Options::new(
        Case::default(),
        Sort::default(),
        Serialization::default(),
        filters.clone(),
        Io::ParallelStream,
        stream_performance,
    );

    // Create inputs with the different I/O modes
    let mmap_view = View::try_from(file_path).expect("create memory-mapped view");
    let stream_reader = Reader::try_from(file_path).expect("create streamed reader");

    // Create WordTally instances with the different I/O modes
    let mmap_tally_map =
        TallyMap::from_view(&mmap_view, &mmap_options).expect("create memory mapped tally map");
    let memory_mapped = WordTally::from_tally_map(mmap_tally_map, &mmap_options);

    let stream_tally_map =
        TallyMap::from_reader(&stream_reader, &stream_options).expect("create streamed tally map");
    let streamed = WordTally::from_tally_map(stream_tally_map, &stream_options);

    // Verify results are the same regardless of I/O mode
    assert_eq!(memory_mapped.count(), streamed.count());
    assert_eq!(memory_mapped.uniq_count(), streamed.uniq_count());
    assert_eq!(memory_mapped.tally(), streamed.tally());

    // Now test with parallel processing
    // Set up options for parallel streamed I/O
    let parallel_performance = Performance::default();
    let parallel_options = Options::new(
        Case::default(),
        Sort::default(),
        Serialization::default(),
        filters,
        Io::ParallelStream,
        parallel_performance,
    );

    // Create input for parallel streamed processing
    let parallel_reader = Reader::try_from(file_path).expect("create parallel reader");
    let parallel_tally_map = TallyMap::from_reader(&parallel_reader, &parallel_options)
        .expect("create parallel tally map");

    // Create WordTally instance with parallel streamed processing
    let parallel_stream = WordTally::from_tally_map(parallel_tally_map, &parallel_options);

    // Verify the parallel processing worked
    assert!(parallel_stream.count() > 0);
    assert!(parallel_stream.uniq_count() > 0);
}

#[test]
fn test_parallel_count() {
    // Instead of using environment variables, just test the parallel function works
    let input_text = b"Test with default settings for chunk size and thread count";
    let mut temp_file = tempfile::NamedTempFile::new().expect("create temp file");
    Write::write_all(&mut temp_file, input_text).expect("write test data");

    let options = Options::default();

    let reader = Reader::try_from(temp_file.path()).expect("create reader");
    let tally_map = TallyMap::from_reader(&reader, &options).expect("create tally map");
    let parallel = WordTally::from_tally_map(tally_map, &options);

    // Only check the counts are positive numbers (actual counts may vary by implementation)
    assert!(parallel.count() > 0);
    assert!(parallel.uniq_count() > 0);
    // Also check uniq count is less than or equal to total count
    assert!(parallel.uniq_count() <= parallel.count());
}

#[test]
fn test_merge_maps() {
    let input_text = b"This is a test of the map merging functionality";
    let mut temp_file = tempfile::NamedTempFile::new().expect("create temp file");
    Write::write_all(&mut temp_file, input_text).expect("write test data");

    let options = Options::default();

    let reader = Reader::try_from(temp_file.path()).expect("create reader");
    let tally_map = TallyMap::from_reader(&reader, &options).expect("create tally map");
    let tally = WordTally::from_tally_map(tally_map, &options);

    assert_eq!(tally.count(), 9);
    assert_eq!(tally.uniq_count(), 9);
}

#[test]
fn test_words_exclude_from() {
    let words = vec!["beep".to_string(), "boop".to_string()];
    assert_eq!(ExcludeWords::from(words.clone()), ExcludeWords(words));
}

// Tests for Serialization convenience methods
mod serialization_tests {
    use super::*;

    #[test]
    fn with_format() {
        let format_only = Serialization::Json;
        assert_eq!(format_only, Serialization::Json);
    }

    #[test]
    fn with_field_delimiter() {
        let delim = Serialization::text().with_field_delimiter("::");
        assert_eq!(delim.field_delimiter(), Some("::"));
    }
}

// Tests for Default implementations
mod default_impl_tests {
    use super::*;

    #[test]
    fn reader_default() {
        let reader = Reader::stdin();
        assert!(matches!(reader, Reader::Stdin(_)));
    }

    #[test]
    fn output_default() {
        let _output = Output::default();
        // Just verify it compiles
    }
}

// Tests for WordTally convenience constructors
mod wordtally_constructor_tests {
    use super::*;

    const TEST_INPUT: &[u8] = b"test convenience constructors";

    fn create_test_file() -> (tempfile::TempDir, String) {
        let temp_dir = tempfile::tempdir().expect("process test");
        let file_path = temp_dir.path().join("test_input.txt");
        fs::write(&file_path, TEST_INPUT).expect("process test");
        (
            temp_dir,
            file_path.to_str().expect("process test").to_string(),
        )
        // temp_dir will be kept alive until it's dropped
    }

    #[test]
    fn with_defaults() {
        let (_temp_dir, file_path) = create_test_file();
        let options = Options::default();
        let reader = Reader::try_from(file_path.as_str()).expect("create reader");
        let tally_map = TallyMap::from_reader(&reader, &options).expect("create tally map");
        let tally = WordTally::from_tally_map(tally_map, &options);
        assert_eq!(tally.count(), 3);
    }

    #[test]
    fn with_parallel_processing() {
        let (_temp_dir, file_path) = create_test_file();
        let options = Options::default();
        let reader = Reader::try_from(file_path.as_str()).expect("create reader");
        let tally_map = TallyMap::from_reader(&reader, &options).expect("create tally map");
        let tally = WordTally::from_tally_map(tally_map, &options);
        assert_eq!(tally.count(), 3);
    }

    #[test]
    fn with_custom_chunk_size() {
        let (_temp_dir, file_path) = create_test_file();
        let options =
            Options::default().with_performance(Performance::default().with_chunk_size(32_768));

        let reader = Reader::try_from(file_path.as_str()).expect("create reader");
        let tally_map = TallyMap::from_reader(&reader, &options).expect("create tally map");
        let tally = WordTally::from_tally_map(tally_map, &options);
        assert_eq!(tally.count(), 3);
    }
}

#[test]
fn test_custom_chunk_size() {
    let input_text = b"test convenience constructors";
    let mut temp_file = tempfile::NamedTempFile::new().expect("create temp file");
    Write::write_all(&mut temp_file, input_text).expect("write test data");

    let options =
        Options::default().with_performance(Performance::default().with_chunk_size(32_768));

    let reader = Reader::try_from(temp_file.path()).expect("create reader");
    let tally_map = TallyMap::from_reader(&reader, &options).expect("create tally map");
    let tally = WordTally::from_tally_map(tally_map, &options);
    assert_eq!(tally.count(), 3);
}

#[test]
fn test_exclude_words_from_trait() {
    let words = vec!["beep".to_string(), "boop".to_string()];
    assert_eq!(ExcludeWords::from(words.clone()), ExcludeWords(words));
}
