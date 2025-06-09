use std::collections::HashMap as StdHashMap;
use std::io::Write;
use std::sync::Arc;

use hashbrown::HashMap;
use tempfile::NamedTempFile;
use word_tally::{Count, Options, Reader, Sort, TallyMap, Word, WordTally};

fn create_test_tally_with_text(text: &[u8], sort: Sort) -> WordTally {
    let mut temp_file = NamedTempFile::new().expect("create temp file");
    temp_file.write_all(text).expect("write test data");

    let options = Options::default().with_sort(sort);

    let reader = Reader::try_from(temp_file.path()).expect("create reader");
    let tally_map = TallyMap::from_reader(&reader, &options).expect("create tally map");
    WordTally::from_tally_map(tally_map, &options)
}

#[test]
fn test_into_tally() {
    let input_text = b"Hope is the thing with feathers that perches in the soul";
    let mut temp_file = NamedTempFile::new().expect("create temp file");
    Write::write_all(&mut temp_file, input_text).expect("write test data");

    let options = Arc::new(Options::default());
    let reader = Reader::try_from(temp_file.path()).expect("create reader");
    let tally_map = TallyMap::from_reader(&reader, &options).expect("create tally map");
    let word_tally = WordTally::from_tally_map(tally_map, &options);

    // Use `tally()` to get a reference to the slice.
    let tally = word_tally.tally();

    let mut expected_counts = HashMap::new();
    expected_counts.insert("the", 2);
    expected_counts.insert("Hope", 1);
    expected_counts.insert("is", 1);
    expected_counts.insert("thing", 1);
    expected_counts.insert("with", 1);
    expected_counts.insert("feathers", 1);
    expected_counts.insert("that", 1);
    expected_counts.insert("perches", 1);
    expected_counts.insert("in", 1);
    expected_counts.insert("soul", 1);

    assert_eq!(tally.len(), expected_counts.len());
    for (word, count) in tally {
        let expected_count = expected_counts.get(word.as_ref()).expect("unexpected word");
        assert_eq!(count, expected_count);
    }
}

#[test]
fn test_iterator() {
    let input_text = b"double trouble double";
    let mut temp_file = NamedTempFile::new().expect("create temp file");
    Write::write_all(&mut temp_file, input_text).expect("write test data");

    let options = Arc::new(Options::default());
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
    let mut temp_file = NamedTempFile::new().expect("create temp file");
    Write::write_all(&mut temp_file, input_text).expect("write test data");

    let options = Arc::new(Options::default());
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
            other => unreachable!("unexpected word: {other}"),
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

#[test]
fn test_from_trait_into_hashmap() {
    let input_text = b"perennial amaranth perennial amaranth perennial";
    let tally = create_test_tally_with_text(input_text, Sort::Desc);

    let hashmap_from_tally: StdHashMap<Word, Count> = tally.into();

    assert_eq!(hashmap_from_tally.len(), 2);
    assert_eq!(
        *hashmap_from_tally
            .get("perennial")
            .expect("perennial should be present"),
        3
    );
    assert_eq!(
        *hashmap_from_tally
            .get("amaranth")
            .expect("amaranth should be present"),
        2
    );

    assert_eq!(*hashmap_from_tally.get("perennial").unwrap_or(&0), 3);
    assert_eq!(*hashmap_from_tally.get("amaranth").unwrap_or(&0), 2);
    assert_eq!(*hashmap_from_tally.get("nonexistent").unwrap_or(&0), 0);
}

#[test]
fn test_readme_hashmap_pattern() {
    let input_text = b"seraph celestial seraph celestial seraph";
    let tally = create_test_tally_with_text(input_text, Sort::Desc);

    let lookup: StdHashMap<_, _> = tally.into();

    assert_eq!(lookup.len(), 2);
    assert_eq!(*lookup.get("seraph").unwrap_or(&0), 3);
    assert_eq!(*lookup.get("celestial").unwrap_or(&0), 2);
    assert_eq!(*lookup.get("nonexistent").unwrap_or(&0), 0);
}
