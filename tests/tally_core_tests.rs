use std::io::Write;

use hashbrown::{HashMap, HashSet};
use tempfile::NamedTempFile;
use word_tally::{
    Case, Count, Filters, Io, Options, Performance, Reader, Serialization, Sort, TallyMap,
    WordTally,
};

fn create_test_data_file() -> NamedTempFile {
    let mut temp_file = NamedTempFile::new().expect("create temp file");
    let content = b"A narrow Fellow in the Grass\n\
Occasionally rides -\n\
You may have met him? Did you not\n\
His notice instant is -\n\
\n\
The Grass divides as with a Comb -\n\
A spotted Shaft is seen -\n\
And then it closes at your feet\n\
And opens further on -";
    Write::write_all(&mut temp_file, content).expect("write test data");
    temp_file
}

fn word_tally(case: Case, sort: Sort, serialization: Serialization, filters: Filters) -> WordTally {
    let test_file = Box::leak(Box::new(create_test_data_file()));
    let file_path = test_file.path().to_str().expect("temp file path");

    let options = Options::new(
        case,
        sort,
        serialization,
        filters,
        Io::ParallelStream,
        Performance::default(),
    );

    let options_static = Box::leak(Box::new(options));

    let reader = Reader::try_from(file_path).expect("create reader");
    let tally_map = TallyMap::from_reader(&reader, options_static).expect("create tally map");
    WordTally::from_tally_map(tally_map, options_static)
}

fn word_tally_test(case: Case, sort: Sort, filters: Filters, fields: &ExpectedFields<'_>) {
    let serialization = Serialization::default();
    let word_tally = word_tally(case, sort, serialization, filters);
    assert_eq!(word_tally.count(), fields.count);
    assert_eq!(word_tally.uniq_count(), fields.uniq_count);

    if sort == Sort::Unsorted {
        let expected_set: HashSet<(&str, Count)> = fields.tally.iter().copied().collect();
        let actual_set: HashSet<(&str, Count)> = word_tally
            .tally()
            .iter()
            .map(|(word, count)| (word.as_ref(), *count))
            .collect();
        assert_eq!(expected_set, actual_set);
    } else {
        let actual_tally = word_tally.tally();

        let expected_counts: Vec<Count> = fields.tally.iter().map(|&(_, count)| count).collect();
        let actual_counts: Vec<Count> = actual_tally
            .iter()
            .take(fields.tally.len())
            .map(|(_, count)| *count)
            .collect();

        assert_eq!(expected_counts, actual_counts);

        let mut expected_by_count: HashMap<Count, Vec<&str>> = HashMap::new();
        for &(word, count) in &fields.tally {
            expected_by_count.entry(count).or_default().push(word);
        }

        let mut actual_by_count: HashMap<Count, Vec<Box<str>>> = HashMap::new();
        for (word, count) in actual_tally {
            actual_by_count
                .entry(*count)
                .or_default()
                .push(word.clone());
        }

        for (count, expected_words) in expected_by_count {
            if let Some(actual_words) = actual_by_count.get(&count) {
                let actual_set: HashSet<&str> = actual_words.iter().map(AsRef::as_ref).collect();
                for expected_word in expected_words {
                    assert!(
                        actual_set.contains(expected_word),
                        "Expected word '{expected_word}' not found at count {count}. Actual words: {actual_words:?}"
                    );
                }
            }
        }
    }
}

#[test]
fn lower_case_desc_order() {
    word_tally_test(
        Case::Lower,
        Sort::Desc,
        Filters::default(),
        &ExpectedFields {
            count: 43,
            uniq_count: 36,
            tally: vec![
                ("a", 3),
                ("the", 2),
                ("grass", 2),
                ("you", 2),
                ("is", 2),
                ("and", 2),
            ],
        },
    );
}

#[test]
fn min_char_count_at_max() {
    word_tally_test(
        Case::Lower,
        Sort::Desc,
        Filters::default().with_min_chars(6),
        &ExpectedFields {
            count: 9,
            uniq_count: 9,
            tally: vec![
                ("narrow", 1),
                ("fellow", 1),
                ("occasionally", 1),
                ("notice", 1),
                ("instant", 1),
                ("divides", 1),
                ("spotted", 1),
                ("closes", 1),
                ("further", 1),
            ],
        },
    );
}

#[test]
fn min_word_count_at_max() {
    word_tally_test(
        Case::Lower,
        Sort::Desc,
        Filters::default().with_min_count(2),
        &ExpectedFields {
            count: 13, // Total count of words that appear 2+ times
            uniq_count: 6,
            tally: vec![
                ("a", 3),
                ("the", 2),
                ("grass", 2),
                ("you", 2),
                ("is", 2),
                ("and", 2),
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
            count: 43,
            uniq_count: 39,
            tally: vec![
                ("narrow", 1),
                ("Fellow", 1),
                ("in", 1),
                ("the", 1),
                ("Occasionally", 1),
                ("rides", 1),
                ("You", 1),
                ("may", 1),
                ("have", 1),
                ("met", 1),
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
            count: 43,
            uniq_count: 36,
            tally: vec![
                ("A", 3),
                ("THE", 2),
                ("GRASS", 2),
                ("YOU", 2),
                ("IS", 2),
                ("AND", 2),
            ],
        },
    );
}

#[derive(Debug)]
struct ExpectedFields<'a> {
    count: Count,
    uniq_count: Count,
    tally: Vec<(&'a str, Count)>,
}

fn create_test_tally_with_text(input_text: &[u8], sort: Sort) -> WordTally {
    let mut temp_file = NamedTempFile::new().expect("create temp file");
    Write::write_all(&mut temp_file, input_text).expect("write test data");
    let temp_file_static = Box::leak(Box::new(temp_file));
    let file_path = temp_file_static.path().to_str().expect("temp file path");

    let options = Options::default().with_sort(sort);
    let options_static = Box::leak(Box::new(options));

    let reader = Reader::try_from(file_path).expect("create reader");
    let tally_map = TallyMap::from_reader(&reader, options_static).expect("create tally map");
    WordTally::from_tally_map(tally_map, options_static)
}

#[test]
fn test_sort_mutates_tally() {
    let input_text = b"revery circuit narrow revery circuit revery narrow narrow narrow narrow";

    // Test ascending sort
    let tally_asc = create_test_tally_with_text(input_text, Sort::Asc);
    let asc_tally = tally_asc.tally().to_vec();
    assert_eq!(asc_tally[0].0.as_ref(), "circuit");
    assert_eq!(asc_tally[0].1, 2);

    // Test descending sort
    let tally_desc = create_test_tally_with_text(input_text, Sort::Desc);
    let desc_tally = tally_desc.tally().to_vec();
    assert_eq!(desc_tally[0].0.as_ref(), "narrow");
    assert_eq!(desc_tally[0].1, 5);

    // Test unsorted has all expected words regardless of order
    let tally_unsorted = create_test_tally_with_text(input_text, Sort::Unsorted);
    assert_eq!(tally_unsorted.count(), 10);
    assert_eq!(tally_unsorted.uniq_count(), 3);

    // Verify all words are present with correct counts
    let unsorted_map: HashMap<_, _> = tally_unsorted
        .tally()
        .iter()
        .map(|(word, count)| (word.as_ref(), *count))
        .collect();
    assert_eq!(unsorted_map.get("narrow"), Some(&5));
    assert_eq!(unsorted_map.get("revery"), Some(&3));
    assert_eq!(unsorted_map.get("circuit"), Some(&2));
}

#[test]
fn test_tally_with_punctuation() {
    let input_text = b"Hello, world! Hello world. World? World!";
    let tally = create_test_tally_with_text(input_text, Sort::Desc);

    assert_eq!(tally.count(), 6);
    assert_eq!(tally.uniq_count(), 3);

    let tally_vec = tally.tally().to_vec();
    // All words have count 2, so we check they exist with correct counts
    let word_counts: HashMap<_, _> = tally_vec
        .iter()
        .map(|(word, count)| (word.as_ref(), *count))
        .collect();

    assert_eq!(word_counts.get("Hello"), Some(&2));
    assert_eq!(word_counts.get("World"), Some(&2));
    assert_eq!(word_counts.get("world"), Some(&2));
}

#[test]
fn test_large_input() {
    let mut input = Vec::new();
    let words = ["Eternity", "Immortal", "Beyond", "Forever", "Infinite"];
    for i in 0..1000 {
        let word = words[i % words.len()];
        input.extend_from_slice(word.as_bytes());
        input.push(b' ');
    }

    let tally = create_test_tally_with_text(&input, Sort::Desc);

    assert_eq!(tally.count(), 1000);
    assert_eq!(tally.uniq_count(), 5);

    let tally_vec = tally.tally().to_vec();
    for item in tally_vec {
        assert_eq!(item.1, 200);
    }
}

#[test]
fn test_edge_case_empty() {
    let input_text = b"";
    let tally = create_test_tally_with_text(input_text, Sort::Desc);

    assert_eq!(tally.count(), 0);
    assert_eq!(tally.uniq_count(), 0);
    assert!(tally.tally().is_empty());
}

#[test]
fn test_edge_case_only_whitespace() {
    let input_text = b"    \n\t\r\n  ";
    let tally = create_test_tally_with_text(input_text, Sort::Desc);

    assert_eq!(tally.count(), 0);
    assert_eq!(tally.uniq_count(), 0);
    assert!(tally.tally().is_empty());
}

#[test]
fn test_edge_case_single_word() {
    let input_text = b"Nobody";
    let tally = create_test_tally_with_text(input_text, Sort::Desc);

    assert_eq!(tally.count(), 1);
    assert_eq!(tally.uniq_count(), 1);

    let tally_vec = tally.tally().to_vec();
    assert_eq!(tally_vec[0].0.as_ref(), "Nobody");
    assert_eq!(tally_vec[0].1, 1);
}

#[test]
fn test_numeric_sorting() {
    let input_text = b"10 100 20 30 200 1 2 3 11";
    let tally = create_test_tally_with_text(input_text, Sort::Desc);

    assert_eq!(tally.count(), 9);
    assert_eq!(tally.uniq_count(), 9);

    let tally_vec = tally.tally().to_vec();
    assert_eq!(
        tally_vec
            .iter()
            .find(|(word, _)| word.as_ref() == "1")
            .expect("execute operation")
            .1,
        1
    );
    assert_eq!(
        tally_vec
            .iter()
            .find(|(word, _)| word.as_ref() == "200")
            .expect("execute operation")
            .1,
        1
    );
}

#[test]
fn test_null_bytes() {
    let input_text = b"word1\0word2\0word3";
    let tally = create_test_tally_with_text(input_text, Sort::Desc);

    // Null bytes don't split words, so we get 3 separate words
    assert_eq!(tally.count(), 3);
    assert_eq!(tally.uniq_count(), 3);

    let tally_vec = tally.tally().to_vec();
    assert_eq!(tally_vec.len(), 3);
}

#[test]
fn test_mixed_line_endings() {
    let input_text = b"word1\nword2\r\nword3\rword4";
    let tally = create_test_tally_with_text(input_text, Sort::Desc);

    assert_eq!(tally.count(), 4);
    assert_eq!(tally.uniq_count(), 4);
}
