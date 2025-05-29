//! Tests for hash functionality.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use word_tally::{
    Case, Filters, Format, Input, Io, Options, Serialization, Sort, Tally, WordTally,
};

fn calculate_hash<T: Hash>(value: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

#[test]
fn test_wordtally_hash() {
    let hope_text = "Hope is the thing with";
    let base_options = Options::default();

    let hope_input_alpha = Input::from_bytes(hope_text);
    let hope_input_beta = Input::from_bytes(hope_text);

    let hope_tally_alpha =
        WordTally::new(&hope_input_alpha, &base_options).expect("create word tally");
    let hope_tally_beta =
        WordTally::new(&hope_input_beta, &base_options).expect("create word tally");

    assert_eq!(hope_tally_alpha.count(), hope_tally_beta.count());
    assert_eq!(hope_tally_alpha.uniq_count(), hope_tally_beta.uniq_count());

    let extended_hope_text = "Hope is the thing with feathers That perches";
    let extended_hope_input = Input::from_bytes(extended_hope_text);
    let extended_hope_tally =
        WordTally::new(&extended_hope_input, &base_options).expect("create word tally");

    assert_ne!(hope_tally_alpha.count(), extended_hope_tally.count());
    assert_ne!(
        hope_tally_alpha.uniq_count(),
        extended_hope_tally.uniq_count()
    );

    let repeated_hope_text = "Hope is the thing with Hope is the";
    let repeated_hope_input = Input::from_bytes(repeated_hope_text);
    let repeated_hope_tally =
        WordTally::new(&repeated_hope_input, &base_options).expect("create word tally");

    assert_ne!(hope_tally_alpha.count(), repeated_hope_tally.count());
}

#[test]
fn test_options_hash() {
    let lowercase_desc_alpha = Options::default()
        .with_case(Case::Lower)
        .with_sort(Sort::Desc);

    let lowercase_desc_beta = Options::default()
        .with_case(Case::Lower)
        .with_sort(Sort::Desc);

    assert_eq!(
        calculate_hash(&lowercase_desc_alpha),
        calculate_hash(&lowercase_desc_beta)
    );

    let uppercase_desc = Options::default()
        .with_case(Case::Upper)
        .with_sort(Sort::Desc);

    assert_ne!(
        calculate_hash(&lowercase_desc_alpha),
        calculate_hash(&uppercase_desc)
    );

    let lowercase_desc_json = Options::default()
        .with_case(Case::Lower)
        .with_sort(Sort::Desc)
        .with_format(Format::Json);

    assert_ne!(
        calculate_hash(&lowercase_desc_alpha),
        calculate_hash(&lowercase_desc_json)
    );

    let lowercase_desc_in_memory = Options::default()
        .with_case(Case::Lower)
        .with_sort(Sort::Desc)
        .with_io(Io::ParallelInMemory);

    assert_ne!(
        calculate_hash(&lowercase_desc_alpha),
        calculate_hash(&lowercase_desc_in_memory)
    );
}

#[test]
fn test_filters_hash() {
    let three_chars_twice_alpha = Filters::default().with_min_chars(3).with_min_count(2);

    let three_chars_twice_beta = Filters::default().with_min_chars(3).with_min_count(2);

    assert_eq!(
        calculate_hash(&three_chars_twice_alpha),
        calculate_hash(&three_chars_twice_beta)
    );

    let four_chars_twice = Filters::default().with_min_chars(4).with_min_count(2);

    assert_ne!(
        calculate_hash(&three_chars_twice_alpha),
        calculate_hash(&four_chars_twice)
    );

    let three_chars_twice_no_articles = Filters::default()
        .with_min_chars(3)
        .with_min_count(2)
        .with_exclude_words(vec!["the".to_string(), "a".to_string()]);

    assert_ne!(
        calculate_hash(&three_chars_twice_alpha),
        calculate_hash(&three_chars_twice_no_articles)
    );
}

#[test]
fn test_components_hash() {
    let lowercase_alpha = Case::Lower;
    let lowercase_beta = Case::Lower;
    let uppercase = Case::Upper;

    assert_eq!(
        calculate_hash(&lowercase_alpha),
        calculate_hash(&lowercase_beta)
    );
    assert_ne!(calculate_hash(&lowercase_alpha), calculate_hash(&uppercase));

    let descending_alpha = Sort::Desc;
    let descending_beta = Sort::Desc;
    let ascending = Sort::Asc;

    assert_eq!(
        calculate_hash(&descending_alpha),
        calculate_hash(&descending_beta)
    );
    assert_ne!(
        calculate_hash(&descending_alpha),
        calculate_hash(&ascending)
    );

    let in_memory_alpha = Io::ParallelInMemory;
    let in_memory_beta = Io::ParallelInMemory;
    let streamed = Io::ParallelStream;

    assert_eq!(
        calculate_hash(&in_memory_alpha),
        calculate_hash(&in_memory_beta)
    );
    assert_ne!(calculate_hash(&in_memory_alpha), calculate_hash(&streamed));

    let text_alpha = Format::Text;
    let text_beta = Format::Text;
    let json = Format::Json;

    assert_eq!(calculate_hash(&text_alpha), calculate_hash(&text_beta));
    assert_ne!(calculate_hash(&text_alpha), calculate_hash(&json));
}

#[test]
fn test_tally_hash() {
    let hello_world_alpha: Tally = Box::new([("hello".into(), 5), ("world".into(), 3)]);

    let hello_world_beta: Tally = Box::new([("hello".into(), 5), ("world".into(), 3)]);

    let hello_worlds: Tally = Box::new([("hello".into(), 5), ("world".into(), 4)]);

    assert_eq!(
        calculate_hash(&hello_world_alpha),
        calculate_hash(&hello_world_beta)
    );
    assert_ne!(
        calculate_hash(&hello_world_alpha),
        calculate_hash(&hello_worlds)
    );
}

#[test]
fn test_hash_collisions() {
    let lowercase_asc_text = Options::default()
        .with_case(Case::Lower)
        .with_sort(Sort::Asc)
        .with_format(Format::Text);

    let uppercase_desc_json = Options::default()
        .with_case(Case::Upper)
        .with_sort(Sort::Desc)
        .with_format(Format::Json);

    assert_ne!(
        calculate_hash(&lowercase_asc_text),
        calculate_hash(&uppercase_desc_json)
    );

    let bird_text = "I shall keep singing";
    let truth_text = "Tell all the truth but";

    let bird_input = Input::from_bytes(bird_text);
    let truth_input = Input::from_bytes(truth_text);

    let bird_tally = WordTally::new(&bird_input, &lowercase_asc_text).expect("create word tally");
    let truth_tally =
        WordTally::new(&truth_input, &uppercase_desc_json).expect("create word tally");

    assert_ne!(calculate_hash(&bird_tally), calculate_hash(&truth_tally));
}

#[test]
fn test_wordtally_includes_options_in_hash() {
    let success_text = "Success is counted sweetest";

    let lowercase_options = Options::default().with_case(Case::Lower);

    let uppercase_options = Options::default().with_case(Case::Upper);

    let success_input_alpha = Input::from_bytes(success_text);
    let success_input_beta = Input::from_bytes(success_text);

    let success_lowercase =
        WordTally::new(&success_input_alpha, &lowercase_options).expect("create word tally");
    let success_uppercase =
        WordTally::new(&success_input_beta, &uppercase_options).expect("create word tally");

    assert_ne!(success_lowercase, success_uppercase);

    let lowercase_asc = Options::default()
        .with_case(Case::Lower)
        .with_sort(Sort::Asc);

    let lowercase_desc = Options::default()
        .with_case(Case::Lower)
        .with_sort(Sort::Desc);

    let success_input_gamma = Input::from_bytes(success_text);
    let success_input_delta = Input::from_bytes(success_text);

    let success_ascending =
        WordTally::new(&success_input_gamma, &lowercase_asc).expect("create word tally");
    let success_descending =
        WordTally::new(&success_input_delta, &lowercase_desc).expect("create word tally");

    assert_ne!(success_ascending, success_descending);
}

#[test]
fn test_wordtally_hash_fields() {
    let text = "The Brain is wider than";
    let input = Input::from_bytes(text);
    let options = Options::default();
    let tally = WordTally::new(&input, &options).expect("create word tally");

    let doubled_text = "The Brain is wider than the Sky For put";
    let doubled_input = Input::from_bytes(doubled_text);
    let doubled_tally = WordTally::new(&doubled_input, &options).expect("create word tally");

    assert_ne!(tally, doubled_tally);

    let uppercase_options = Options::default().with_case(Case::Upper);
    let uppercase_input = Input::from_bytes(text);
    let uppercase_tally =
        WordTally::new(&uppercase_input, &uppercase_options).expect("create word tally");

    assert_ne!(tally, uppercase_tally);
}

#[test]
fn test_equality_and_hashing() {
    fn assert_hash_eq(tally_a: &WordTally<'_>, tally_b: &WordTally<'_>) {
        assert_eq!(tally_a, tally_b);
    }

    fn assert_hash_ne(tally_a: &WordTally<'_>, tally_b: &WordTally<'_>) {
        assert_ne!(tally_a, tally_b);
    }

    let cases_and_sorts = [
        (Case::Original, Sort::Asc),
        (Case::Original, Sort::Desc),
        (Case::Upper, Sort::Asc),
        (Case::Upper, Sort::Desc),
        (Case::Lower, Sort::Asc),
        (Case::Lower, Sort::Desc),
    ];

    // Create a test file
    let test_text = b"test text for hashing";
    let mut temp_file = tempfile::NamedTempFile::new().expect("create temp file");
    std::io::Write::write_all(&mut temp_file, test_text).expect("write test data");
    let file_path = temp_file.path().to_str().expect("temp file path");

    let tallies: Vec<WordTally<'static>> = cases_and_sorts
        .iter()
        .map(|&(case, sort)| {
            let serializer = Serialization::with_format(Format::Text);
            let filters = Filters::default();
            let options = Options::new(
                case,
                sort,
                serializer,
                filters,
                Io::ParallelStream,
                word_tally::Performance::default(),
            );
            // For tests only: create a 'static reference using Box::leak
            let options_static = Box::leak(Box::new(options));

            let input = Input::new(file_path, options_static.io()).expect("Failed to create Input");

            WordTally::new(&input, options_static).expect("Failed to create WordTally")
        })
        .collect();

    for tally in &tallies {
        assert_eq!(tally, tally);
        assert_hash_eq(tally, tally);
    }

    for (i, tally_a) in tallies.iter().enumerate() {
        for (j, tally_b) in tallies.iter().enumerate() {
            if i != j {
                assert_ne!(tally_a, tally_b);
                assert_hash_ne(tally_a, tally_b);
            }
        }
    }
}
