use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use word_tally::{Case, Filters, Format, Input, Io, Options, Processing, Sort, Tally, WordTally};

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

    let hope_tally_alpha = WordTally::new(&hope_input_alpha, &base_options).unwrap();
    let hope_tally_beta = WordTally::new(&hope_input_beta, &base_options).unwrap();

    assert_eq!(
        calculate_hash(&hope_tally_alpha),
        calculate_hash(&hope_tally_beta)
    );

    let extended_hope_text = "Hope is the thing with feathers That perches";
    let extended_hope_input = Input::from_bytes(extended_hope_text);
    let extended_hope_tally = WordTally::new(&extended_hope_input, &base_options).unwrap();

    assert_ne!(
        calculate_hash(&hope_tally_alpha),
        calculate_hash(&extended_hope_tally)
    );

    let repeated_hope_text = "Hope is the thing with Hope is the";
    let repeated_hope_input = Input::from_bytes(repeated_hope_text);
    let repeated_hope_tally = WordTally::new(&repeated_hope_input, &base_options).unwrap();

    assert_ne!(
        calculate_hash(&hope_tally_alpha),
        calculate_hash(&repeated_hope_tally)
    );
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

    let lowercase_desc_parallel = Options::default()
        .with_case(Case::Lower)
        .with_sort(Sort::Desc)
        .with_processing(Processing::Parallel)
        .with_io(Io::Buffered);

    assert_ne!(
        calculate_hash(&lowercase_desc_alpha),
        calculate_hash(&lowercase_desc_parallel)
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

    let buffered_alpha = Io::Buffered;
    let buffered_beta = Io::Buffered;
    let streamed = Io::Streamed;

    assert_eq!(
        calculate_hash(&buffered_alpha),
        calculate_hash(&buffered_beta)
    );
    assert_ne!(calculate_hash(&buffered_alpha), calculate_hash(&streamed));

    let sequential_alpha = Processing::Sequential;
    let sequential_beta = Processing::Sequential;
    let parallel = Processing::Parallel;

    assert_eq!(
        calculate_hash(&sequential_alpha),
        calculate_hash(&sequential_beta)
    );
    assert_ne!(calculate_hash(&sequential_alpha), calculate_hash(&parallel));

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

    let bird_tally = WordTally::new(&bird_input, &lowercase_asc_text).unwrap();
    let truth_tally = WordTally::new(&truth_input, &uppercase_desc_json).unwrap();

    assert_ne!(calculate_hash(&bird_tally), calculate_hash(&truth_tally));
}

#[test]
fn test_wordtally_includes_options_in_hash() {
    let success_text = "Success is counted sweetest";

    let lowercase_options = Options::default().with_case(Case::Lower);

    let uppercase_options = Options::default().with_case(Case::Upper);

    let success_input_alpha = Input::from_bytes(success_text);
    let success_input_beta = Input::from_bytes(success_text);

    let success_lowercase = WordTally::new(&success_input_alpha, &lowercase_options).unwrap();
    let success_uppercase = WordTally::new(&success_input_beta, &uppercase_options).unwrap();

    assert_ne!(
        calculate_hash(&success_lowercase),
        calculate_hash(&success_uppercase)
    );

    let lowercase_asc = Options::default()
        .with_case(Case::Lower)
        .with_sort(Sort::Asc);

    let lowercase_desc = Options::default()
        .with_case(Case::Lower)
        .with_sort(Sort::Desc);

    let success_input_gamma = Input::from_bytes(success_text);
    let success_input_delta = Input::from_bytes(success_text);

    let success_ascending = WordTally::new(&success_input_gamma, &lowercase_asc).unwrap();
    let success_descending = WordTally::new(&success_input_delta, &lowercase_desc).unwrap();

    assert_ne!(
        calculate_hash(&success_ascending),
        calculate_hash(&success_descending)
    );
}

#[test]
fn test_wordtally_hash_fields() {
    let text = "The Brain is wider than";
    let input = Input::from_bytes(text);
    let options = Options::default();
    let tally = WordTally::new(&input, &options).unwrap();

    let initial_hash = calculate_hash(&tally);

    let doubled_text = "The Brain is wider than the Sky For put";
    let doubled_input = Input::from_bytes(doubled_text);
    let doubled_tally = WordTally::new(&doubled_input, &options).unwrap();

    assert_ne!(initial_hash, calculate_hash(&doubled_tally));

    let uppercase_options = Options::default().with_case(Case::Upper);
    let uppercase_input = Input::from_bytes(text);
    let uppercase_tally = WordTally::new(&uppercase_input, &uppercase_options).unwrap();

    assert_ne!(initial_hash, calculate_hash(&uppercase_tally));
}
