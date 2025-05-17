use std::io::Write;
use std::sync::Arc;
use tempfile::NamedTempFile;
use word_tally::{Count, Input, Options, Sort, Word, WordTally};

fn make_shared<T>(value: T) -> Arc<T> {
    Arc::new(value)
}

fn create_test_tally_with_text(text: &[u8], sort: Sort) -> WordTally<'static> {
    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(text).unwrap();

    let options = Options::default().with_sort(sort);
    let options_static = Box::leak(Box::new(options));

    let input = Input::new(temp_file.path().to_str().unwrap(), options_static.io()).unwrap();
    WordTally::new(&input, options_static).unwrap()
}

#[test]
fn test_into_tally() {
    let input_text = b"Hope is the thing with feathers that perches in the soul";
    let mut temp_file = tempfile::NamedTempFile::new().unwrap();
    std::io::Write::write_all(&mut temp_file, input_text).unwrap();

    let options = make_shared(Options::default());
    let input = Input::new(temp_file.path().to_str().unwrap(), options.io())
        .expect("Failed to create Input");

    let word_tally = WordTally::new(&input, &options).expect("Failed to create WordTally");

    // Use `tally()` to get a reference to the slice.
    let tally = word_tally.tally();

    let expected_tally: word_tally::Tally = vec![
        ("the".into(), 2),
        ("hope".into(), 1),
        ("is".into(), 1),
        ("thing".into(), 1),
        ("with".into(), 1),
        ("feathers".into(), 1),
        ("that".into(), 1),
        ("perches".into(), 1),
        ("in".into(), 1),
        ("soul".into(), 1),
    ]
    .into_boxed_slice();

    assert_eq!(tally, expected_tally.as_ref());
}

#[test]
fn test_iterator() {
    let input_text = b"double trouble double";
    let mut temp_file = tempfile::NamedTempFile::new().unwrap();
    std::io::Write::write_all(&mut temp_file, input_text).unwrap();

    let options = make_shared(Options::default());
    let input = Input::new(temp_file.path().to_str().unwrap(), options.io())
        .expect("Failed to create Input");

    let word_tally = WordTally::new(&input, &options).expect("Failed to create WordTally");

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
    let mut temp_file = tempfile::NamedTempFile::new().unwrap();
    std::io::Write::write_all(&mut temp_file, input_text).unwrap();

    let options = make_shared(Options::default());
    let input = Input::new(temp_file.path().to_str().unwrap(), options.io())
        .expect("Failed to create Input");

    let word_tally = WordTally::new(&input, &options).expect("Failed to create WordTally");

    let expected: Vec<(Word, Count)> = vec![(Box::from("llama"), 2), (Box::from("pajamas"), 1)];

    let mut collected = vec![];
    for item in &word_tally {
        collected.push(item.clone());
    }
    assert_eq!(collected, expected);
}

#[test]
fn test_into_tally_consumes() {
    let input_text = b"circumference plash slant bequest";
    let tally = create_test_tally_with_text(input_text, Sort::Desc);

    let count_before = tally.count();
    let uniq_count_before = tally.uniq_count();

    let tally_box = tally.into_tally();

    assert_eq!(tally_box.len(), uniq_count_before);

    let tally_vec: Vec<(Word, Count)> = tally_box.into_vec();
    let total_count: Count = tally_vec.iter().map(|(_, count)| count).sum();
    assert_eq!(total_count, count_before);
}

#[test]
fn test_into_iterator_trait() {
    let input_text = b"phosphor alabaster cochineal phosphor alabaster phosphor";
    let tally = create_test_tally_with_text(input_text, Sort::Desc);

    assert_eq!((&tally).into_iter().count(), 3);

    let mut count = 0;
    for (word, freq) in &tally {
        count += 1;
        match word.as_ref() {
            "phosphor" => assert_eq!(*freq, 3),
            "alabaster" => assert_eq!(*freq, 2),
            "cochineal" => assert_eq!(*freq, 1),
            _ => panic!("Unexpected word"),
        }
    }
    assert_eq!(count, 3);

    let first_iter: Vec<_> = (&tally).into_iter().collect();
    let second_iter: Vec<_> = (&tally).into_iter().collect();
    assert_eq!(first_iter, second_iter);
}

#[test]
fn test_from_trait_into_vec() {
    let input_text = b"diadem syllable finite diadem syllable diadem";
    let tally = create_test_tally_with_text(input_text, Sort::Desc);

    let vec_from_tally: Vec<(Word, Count)> = tally.into();

    assert_eq!(vec_from_tally.len(), 3);
    assert_eq!(vec_from_tally[0].0.as_ref(), "diadem");
    assert_eq!(vec_from_tally[0].1, 3);
    assert_eq!(vec_from_tally[1].0.as_ref(), "syllable");
    assert_eq!(vec_from_tally[1].1, 2);
    assert_eq!(vec_from_tally[2].0.as_ref(), "finite");
    assert_eq!(vec_from_tally[2].1, 1);
}
