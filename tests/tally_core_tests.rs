use tempfile::NamedTempFile;
use word_tally::{
    Case, Count, Filters, Format, Input, Io, Options, Performance, Processing, Serialization, Sort,
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
    std::io::Write::write_all(&mut temp_file, content).expect("write test data");
    temp_file
}

fn word_tally(
    case: Case,
    sort: Sort,
    serialization: Serialization,
    filters: Filters,
) -> WordTally<'static> {
    let test_file = Box::leak(Box::new(create_test_data_file()));
    let file_path = test_file.path().to_str().expect("temp file path");

    let options = Options::new(
        case,
        sort,
        serialization,
        filters,
        Io::Streamed,
        Processing::Sequential,
        Performance::default(),
    );

    let options_static = Box::leak(Box::new(options));

    let input =
        Input::new(file_path, options_static.io()).expect("Failed to create input from test file");

    WordTally::new(&input, options_static).expect("Failed to create WordTally")
}

fn word_tally_test(case: Case, sort: Sort, filters: Filters, fields: &ExpectedFields<'_>) {
    let serialization = Serialization::with_format(Format::Text);
    let word_tally = word_tally(case, sort, serialization, filters);
    assert_eq!(word_tally.count(), fields.count);
    assert_eq!(word_tally.uniq_count(), fields.uniq_count);

    if sort == Sort::Unsorted {
        let expected_set: std::collections::HashSet<(&str, Count)> =
            fields.tally.iter().cloned().collect();
        let actual_set: std::collections::HashSet<(&str, Count)> = word_tally
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

        let mut expected_by_count: std::collections::HashMap<Count, Vec<&str>> =
            std::collections::HashMap::new();
        for &(word, count) in &fields.tally {
            expected_by_count.entry(count).or_default().push(word);
        }

        let mut actual_by_count: std::collections::HashMap<Count, Vec<Box<str>>> =
            std::collections::HashMap::new();
        for (word, count) in actual_tally.iter() {
            actual_by_count
                .entry(*count)
                .or_default()
                .push(word.clone());
        }

        for (count, expected_words) in expected_by_count {
            if let Some(actual_words) = actual_by_count.get(&count) {
                let actual_set: std::collections::HashSet<&str> =
                    actual_words.iter().map(|w| w.as_ref()).collect();
                for expected_word in expected_words {
                    assert!(
                        actual_set.contains(expected_word),
                        "Expected word '{}' not found at count {}. Actual words: {:?}",
                        expected_word,
                        count,
                        actual_words
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

fn create_test_tally_with_text(input_text: &[u8], sort: Sort) -> WordTally<'static> {
    let mut temp_file = NamedTempFile::new().expect("create temp file");
    std::io::Write::write_all(&mut temp_file, input_text).expect("write test data");
    let temp_file_static = Box::leak(Box::new(temp_file));
    let file_path = temp_file_static.path().to_str().expect("temp file path");

    let options = Options::default().with_sort(sort);
    let options_static = Box::leak(Box::new(options));

    let input = Input::new(file_path, options_static.io()).expect("Failed to create input");
    WordTally::new(&input, options_static).expect("Failed to create WordTally")
}

#[test]
fn test_sort_mutates_tally() {
    let input_text = b"revery circuit narrow revery circuit revery narrow narrow narrow narrow";
    let mut tally = create_test_tally_with_text(input_text, Sort::Unsorted);

    tally.sort(Sort::Asc);
    let asc_tally = tally.tally().to_vec();

    assert_eq!(asc_tally[0].0.as_ref(), "circuit");
    assert_eq!(asc_tally[0].1, 2);

    tally.sort(Sort::Desc);
    let desc_tally = tally.tally().to_vec();

    assert_eq!(desc_tally[0].0.as_ref(), "narrow");
    assert_eq!(desc_tally[0].1, 5);

    let unsorted_before = tally.tally().to_vec();
    tally.sort(Sort::Unsorted);
    let unsorted_after = tally.tally().to_vec();

    assert_eq!(unsorted_before, unsorted_after);
}

#[test]
fn test_tally_with_punctuation() {
    let input_text = b"Hello, world! Hello world. World? World!";
    let tally = create_test_tally_with_text(input_text, Sort::Desc);

    assert_eq!(tally.count(), 6);
    assert_eq!(tally.uniq_count(), 2);

    let tally_vec = tally.tally().to_vec();
    assert_eq!(tally_vec[0].0.as_ref(), "world");
    assert_eq!(tally_vec[0].1, 4);
    assert_eq!(tally_vec[1].0.as_ref(), "hello");
    assert_eq!(tally_vec[1].1, 2);
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
    assert_eq!(tally_vec[0].0.as_ref(), "nobody");
    assert_eq!(tally_vec[0].1, 1);
}

#[test]
fn test_unicode_and_emoji() {
    let input_text = "café über señor 🌟 hello 🎉 world 🌟 café".as_bytes();
    let tally = create_test_tally_with_text(input_text, Sort::Desc);

    // Emoji may not be treated as separate words
    assert_eq!(tally.count(), 6);
    assert_eq!(tally.uniq_count(), 5);

    let tally_vec = tally.tally().to_vec();
    assert!(
        tally_vec
            .iter()
            .any(|(word, count)| word.as_ref() == "café" && *count == 2)
    );
    assert!(
        tally_vec
            .iter()
            .any(|(word, count)| word.as_ref() == "über" && *count == 1)
    );
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
