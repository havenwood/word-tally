//! Tests for hash functionality.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use word_tally::{
    Case, Filters, Io, Options, Reader, Serialization, Sort, Tally, TallyMap, View, WordTally,
};

fn calculate_hash<T: Hash>(value: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

#[test]
fn test_wordtally_hash() {
    let hope_text = "Hope is the thing with";
    let base_options = Options::default().with_io(Io::ParallelBytes);

    let hope_view_alpha = View::from(hope_text.as_bytes());
    let hope_view_beta = View::from(hope_text.as_bytes());

    let hope_tally_map_alpha =
        TallyMap::from_view(&hope_view_alpha, &base_options).expect("create tally map");
    let hope_tally_map_beta =
        TallyMap::from_view(&hope_view_beta, &base_options).expect("create tally map");

    let hope_tally_alpha = WordTally::from_tally_map(hope_tally_map_alpha, &base_options);
    let hope_tally_beta = WordTally::from_tally_map(hope_tally_map_beta, &base_options);

    assert_eq!(hope_tally_alpha.count(), hope_tally_beta.count());
    assert_eq!(hope_tally_alpha.uniq_count(), hope_tally_beta.uniq_count());

    let extended_hope_text = "Hope is the thing with feathers That perches";
    let extended_hope_view = View::from(extended_hope_text.as_bytes());
    let extended_hope_tally_map =
        TallyMap::from_view(&extended_hope_view, &base_options).expect("create tally map");
    let extended_hope_tally = WordTally::from_tally_map(extended_hope_tally_map, &base_options);

    assert_ne!(hope_tally_alpha.count(), extended_hope_tally.count());
    assert_ne!(
        hope_tally_alpha.uniq_count(),
        extended_hope_tally.uniq_count()
    );

    let repeated_hope_text = "Hope is the thing with Hope is the";
    let repeated_hope_view = View::from(repeated_hope_text.as_bytes());
    let repeated_hope_tally_map =
        TallyMap::from_view(&repeated_hope_view, &base_options).expect("create tally map");
    let repeated_hope_tally = WordTally::from_tally_map(repeated_hope_tally_map, &base_options);

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
        .with_serialization(Serialization::Json);

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

    let text_alpha = Serialization::default();
    let text_beta = Serialization::default();
    let json = Serialization::Json;

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
        .with_io(Io::ParallelBytes)
        .with_case(Case::Lower)
        .with_sort(Sort::Asc)
        .with_serialization(Serialization::default());

    let uppercase_desc_json = Options::default()
        .with_io(Io::ParallelBytes)
        .with_case(Case::Upper)
        .with_sort(Sort::Desc)
        .with_serialization(Serialization::Json);

    assert_ne!(
        calculate_hash(&lowercase_asc_text),
        calculate_hash(&uppercase_desc_json)
    );

    let bird_text = "I shall keep singing";
    let truth_text = "Tell all the truth but";

    let bird_view = View::from(bird_text.as_bytes());
    let truth_view = View::from(truth_text.as_bytes());

    let bird_tally_map =
        TallyMap::from_view(&bird_view, &lowercase_asc_text).expect("create tally map");
    let bird_tally = WordTally::from_tally_map(bird_tally_map, &lowercase_asc_text);

    let truth_tally_map =
        TallyMap::from_view(&truth_view, &uppercase_desc_json).expect("create tally map");
    let truth_tally = WordTally::from_tally_map(truth_tally_map, &uppercase_desc_json);

    assert_ne!(calculate_hash(&bird_tally), calculate_hash(&truth_tally));
}

#[test]
fn test_wordtally_includes_options_in_hash() {
    let success_text = "Success is counted sweetest";

    let lowercase_options = Options::default()
        .with_io(Io::ParallelBytes)
        .with_case(Case::Lower);

    let uppercase_options = Options::default()
        .with_io(Io::ParallelBytes)
        .with_case(Case::Upper);

    let success_view_alpha = View::from(success_text.as_bytes());
    let success_view_beta = View::from(success_text.as_bytes());

    let success_lowercase_map =
        TallyMap::from_view(&success_view_alpha, &lowercase_options).expect("create tally map");
    let success_lowercase = WordTally::from_tally_map(success_lowercase_map, &lowercase_options);

    let success_uppercase_map =
        TallyMap::from_view(&success_view_beta, &uppercase_options).expect("create tally map");
    let success_uppercase = WordTally::from_tally_map(success_uppercase_map, &uppercase_options);

    assert_ne!(success_lowercase, success_uppercase);

    let lowercase_asc = Options::default()
        .with_io(Io::ParallelBytes)
        .with_case(Case::Lower)
        .with_sort(Sort::Asc);

    let lowercase_desc = Options::default()
        .with_io(Io::ParallelBytes)
        .with_case(Case::Lower)
        .with_sort(Sort::Desc);

    let success_view_gamma = View::from(success_text.as_bytes());
    let success_view_delta = View::from(success_text.as_bytes());

    let success_ascending_map =
        TallyMap::from_view(&success_view_gamma, &lowercase_asc).expect("create tally map");
    let success_ascending = WordTally::from_tally_map(success_ascending_map, &lowercase_asc);

    let success_descending_map =
        TallyMap::from_view(&success_view_delta, &lowercase_desc).expect("create tally map");
    let success_descending = WordTally::from_tally_map(success_descending_map, &lowercase_desc);

    assert_ne!(success_ascending, success_descending);
}

#[test]
fn test_wordtally_hash_fields() {
    let text = "The Brain is wider than";
    let view = View::from(text.as_bytes());
    let options = Options::default().with_io(Io::ParallelBytes);
    let tally_map = TallyMap::from_view(&view, &options).expect("create tally map");
    let tally = WordTally::from_tally_map(tally_map, &options);

    let doubled_text = "The Brain is wider than the Sky For put";
    let doubled_view = View::from(doubled_text.as_bytes());
    let doubled_tally_map = TallyMap::from_view(&doubled_view, &options).expect("create tally map");
    let doubled_tally = WordTally::from_tally_map(doubled_tally_map, &options);

    assert_ne!(tally, doubled_tally);

    let uppercase_options = Options::default()
        .with_io(Io::ParallelBytes)
        .with_case(Case::Upper);
    let uppercase_view = View::from(text.as_bytes());
    let uppercase_tally_map =
        TallyMap::from_view(&uppercase_view, &uppercase_options).expect("create tally map");
    let uppercase_tally = WordTally::from_tally_map(uppercase_tally_map, &uppercase_options);

    assert_ne!(tally, uppercase_tally);
}

#[test]
fn test_equality_and_hashing() {
    fn assert_hash_eq(tally_a: &WordTally, tally_b: &WordTally) {
        assert_eq!(tally_a, tally_b);
    }

    fn assert_hash_ne(tally_a: &WordTally, tally_b: &WordTally) {
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

    let tallies: Vec<WordTally> = cases_and_sorts
        .iter()
        .map(|&(case, sort)| {
            let serializer = Serialization::default();
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

            let reader = Reader::try_from(file_path).expect("create reader");
            let tally_map =
                TallyMap::from_reader(&reader, options_static).expect("create tally map");
            WordTally::from_tally_map(tally_map, options_static)
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
