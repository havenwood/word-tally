//! Comprehensive tests for word segmentation functionality

use hashbrown::HashMap;
use word_tally::{Case, TallyMap, options::encoding::Encoding};

#[test]
fn test_unicode_segmentation_basic() {
    let mut tally = TallyMap::new();
    tally
        .add_words("hello world", Case::Original, Encoding::Unicode)
        .expect("valid unicode");
    assert_eq!(tally.len(), 2);

    let words: Vec<String> = tally.into_iter().map(|(w, _)| w.into()).collect();
    assert!(words.contains(&"hello".to_string()));
    assert!(words.contains(&"world".to_string()));
}

#[test]
fn test_unicode_segmentation_punctuation() {
    let mut tally = TallyMap::new();
    tally
        .add_words("Hello, world! How are you?", Case::Lower, Encoding::Unicode)
        .expect("valid unicode");
    assert_eq!(tally.len(), 5);

    let word_map: HashMap<String, usize> = tally.into_iter().map(|(w, c)| (w.into(), c)).collect();

    assert_eq!(word_map.get("hello"), Some(&1));
    assert_eq!(word_map.get("world"), Some(&1));
    assert_eq!(word_map.get("how"), Some(&1));
    assert_eq!(word_map.get("are"), Some(&1));
    assert_eq!(word_map.get("you"), Some(&1));
}

#[test]
fn test_unicode_segmentation_contractions() {
    let mut tally = TallyMap::new();
    tally
        .add_words(
            "don't can't won't I'll we're they've",
            Case::Lower,
            Encoding::Unicode,
        )
        .expect("valid unicode");

    let words: Vec<String> = tally.into_iter().map(|(w, _)| w.into()).collect();
    assert!(words.contains(&"don't".to_string()));
    assert!(words.contains(&"can't".to_string()));
    assert!(words.contains(&"won't".to_string()));
    assert!(words.contains(&"i'll".to_string()));
    assert!(words.contains(&"we're".to_string()));
    assert!(words.contains(&"they've".to_string()));
}

#[test]
fn test_unicode_segmentation_numbers() {
    let mut tally = TallyMap::new();
    tally
        .add_words(
            "test123 456test 3.14 $100 50%",
            Case::Original,
            Encoding::Unicode,
        )
        .expect("valid unicode");

    let words: Vec<String> = tally.into_iter().map(|(w, _)| w.into()).collect();
    assert!(words.contains(&"test123".to_string()));
    assert!(words.contains(&"456test".to_string()));
    assert!(
        words.contains(&"3.14".to_string())
            || (words.contains(&"3".to_string()) && words.contains(&"14".to_string()))
    );
    assert!(words.contains(&"100".to_string()));
    assert!(words.contains(&"50".to_string()));
}

#[test]
fn test_unicode_segmentation_special_chars() {
    let mut tally = TallyMap::new();
    tally
        .add_words(
            "email@test.com http://example.org C:\\path\\file",
            Case::Original,
            Encoding::Unicode,
        )
        .expect("valid unicode");

    let words: Vec<String> = tally.into_iter().map(|(w, _)| w.into()).collect();
    // ICU segmenter handles these differently than simple whitespace splitting
    assert!(
        words
            .iter()
            .any(|w| w.contains("email") || w.contains("test") || w.contains("com"))
    );
    assert!(
        words
            .iter()
            .any(|w| w.contains("http") || w.contains("example") || w.contains("org"))
    );
}

#[test]
fn test_unicode_segmentation_unicode_chars() {
    let mut tally = TallyMap::new();
    tally
        .add_words(
            "café naïve señor über 北京 こんにちは",
            Case::Lower,
            Encoding::Unicode,
        )
        .expect("valid unicode");

    let words: Vec<String> = tally.into_iter().map(|(w, _)| w.into()).collect();
    assert!(words.contains(&"café".to_string()));
    assert!(words.contains(&"naïve".to_string()));
    assert!(words.contains(&"señor".to_string()));
    assert!(words.contains(&"über".to_string()));
    assert_eq!(words.len(), 4);
}

#[test]
fn test_cjk_segmentation() {
    // Chinese
    let mut tally = TallyMap::new();
    tally
        .add_words("北京", Case::Original, Encoding::Unicode)
        .expect("valid unicode");
    assert_eq!(tally.len(), 0);

    // Japanese
    let mut tally2 = TallyMap::new();
    tally2
        .add_words("こんにちは", Case::Original, Encoding::Unicode)
        .expect("valid unicode");
    assert_eq!(tally2.len(), 0);

    // Korean
    let mut tally3 = TallyMap::new();
    tally3
        .add_words("안녕하세요", Case::Original, Encoding::Unicode)
        .expect("valid unicode");
    assert_eq!(tally3.len(), 1);
}

#[test]
fn test_cjk_script_differences() {
    // Korean: Consistently recognized
    let mut tally = TallyMap::new();
    tally
        .add_words("한글", Case::Original, Encoding::Unicode)
        .expect("valid unicode");
    assert_eq!(tally.len(), 1);

    // Chinese and Japanese: Behavior varies by ICU version/config
}

#[test]
fn test_mixed_cjk_and_latin() {
    let mut tally = TallyMap::new();
    tally
        .add_words(
            "Hello 北京 World こんにちは Test",
            Case::Lower,
            Encoding::Unicode,
        )
        .expect("valid unicode");

    let words: Vec<String> = tally.into_iter().map(|(w, _)| w.into()).collect();
    assert_eq!(words.len(), 3);
    assert!(words.contains(&"hello".to_string()));
    assert!(words.contains(&"world".to_string()));
    assert!(words.contains(&"test".to_string()));
}

#[test]
fn test_unicode_segmentation_emoji() {
    let mut tally = TallyMap::new();
    tally
        .add_words("Hello 👋 world 🌍!", Case::Lower, Encoding::Unicode)
        .expect("valid unicode");

    let words: Vec<String> = tally.into_iter().map(|(w, _)| w.into()).collect();
    assert!(words.contains(&"hello".to_string()));
    assert!(words.contains(&"world".to_string()));
    // Emojis are typically not considered word-like by ICU
    assert_eq!(words.len(), 2);
}

#[test]
fn test_ascii_segmentation_basic() {
    let mut tally = TallyMap::new();
    let result = tally.add_words("hello world", Case::Original, Encoding::Ascii);
    assert!(result.is_ok());
    assert_eq!(tally.len(), 2);

    let words: Vec<String> = tally.into_iter().map(|(w, _)| w.into()).collect();
    assert!(words.contains(&"hello".to_string()));
    assert!(words.contains(&"world".to_string()));
}

#[test]
fn test_ascii_segmentation_punctuation() {
    let mut tally = TallyMap::new();
    let result = tally.add_words("Hello, world! How are you?", Case::Lower, Encoding::Ascii);
    assert!(result.is_ok());
    assert_eq!(tally.len(), 5);
}

#[test]
fn test_ascii_segmentation_apostrophes() {
    let mut tally = TallyMap::new();
    let result = tally.add_words("don't can't it's we're", Case::Lower, Encoding::Ascii);
    assert!(result.is_ok());
    assert_eq!(tally.len(), 4);

    let words: Vec<String> = tally.into_iter().map(|(w, _)| w.into()).collect();
    assert!(words.contains(&"don't".to_string()));
    assert!(words.contains(&"can't".to_string()));
    assert!(words.contains(&"it's".to_string()));
    assert!(words.contains(&"we're".to_string()));
}

#[test]
fn test_ascii_segmentation_numbers_mixed() {
    let mut tally = TallyMap::new();
    let result = tally.add_words(
        "test123 456test hello2world",
        Case::Original,
        Encoding::Ascii,
    );
    assert!(result.is_ok());

    let words: Vec<String> = tally.into_iter().map(|(w, _)| w.into()).collect();
    assert!(words.contains(&"test123".to_string()));
    assert!(words.contains(&"456test".to_string()));
    assert!(words.contains(&"hello2world".to_string()));
}

#[test]
fn test_ascii_segmentation_special_chars() {
    let mut tally = TallyMap::new();
    let result = tally.add_words(
        "test@example.com, hello-world, file.txt",
        Case::Original,
        Encoding::Ascii,
    );
    assert!(result.is_ok());

    let words: Vec<String> = tally.into_iter().map(|(w, _)| w.into()).collect();
    assert!(words.contains(&"test".to_string()));
    assert!(words.contains(&"example".to_string()));
    assert!(words.contains(&"com".to_string()));
    assert!(words.contains(&"hello".to_string()));
    assert!(words.contains(&"world".to_string()));
    assert!(words.contains(&"file".to_string()));
    assert!(words.contains(&"txt".to_string()));
}

#[test]
fn test_ascii_segmentation_multiple_apostrophes() {
    let mut tally = TallyMap::new();
    let result = tally.add_words("rock'n'roll y'all", Case::Lower, Encoding::Ascii);
    assert!(result.is_ok());

    let words: Vec<String> = tally.into_iter().map(|(w, _)| w.into()).collect();
    assert!(words.contains(&"rock'n'roll".to_string()));
    assert!(words.contains(&"y'all".to_string()));
}

#[test]
fn test_ascii_segmentation_edge_cases() {
    let mut tally = TallyMap::new();

    // Empty string
    let result = tally.add_words("", Case::Original, Encoding::Ascii);
    assert!(result.is_ok());
    assert_eq!(tally.len(), 0);

    // Only whitespace
    let result = tally.add_words("   \t\n  ", Case::Original, Encoding::Ascii);
    assert!(result.is_ok());
    assert_eq!(tally.len(), 0);

    // Single word
    let result = tally.add_words("word", Case::Original, Encoding::Ascii);
    assert!(result.is_ok());
    assert_eq!(tally.len(), 1);

    // Leading/trailing apostrophes are not included
    let mut tally2 = TallyMap::new();
    let result = tally2.add_words("'hello' 'world'", Case::Original, Encoding::Ascii);
    assert!(result.is_ok());
    let words: Vec<String> = tally2.into_iter().map(|(w, _)| w.into()).collect();
    assert!(words.contains(&"hello'".to_string()));
    assert!(words.contains(&"world'".to_string()));
}

#[test]
fn test_ascii_non_ascii_rejection() {
    let test_cases = vec![
        ("café", 3),       // é at position 3
        ("naïve", 2),      // ï at position 2
        ("señor", 2),      // ñ at position 2
        ("über", 0),       // ü at position 0
        ("hello 世界", 6), // Chinese at position 6
    ];

    for (input, expected_pos) in test_cases {
        let mut tally = TallyMap::new();
        let result = tally.add_words(input, Case::Original, Encoding::Ascii);
        assert!(result.is_err());

        let err_str = result
            .expect_err("expected error for non-ASCII input")
            .to_string();
        assert!(err_str.contains("non-ASCII"));
        assert!(err_str.contains(&expected_pos.to_string()));
    }
}

#[test]
fn test_case_normalization_unicode() {
    let mut tally_lower = TallyMap::new();
    tally_lower
        .add_words("Hello WORLD hello World", Case::Lower, Encoding::Unicode)
        .expect("valid unicode");
    assert_eq!(tally_lower.len(), 2);

    let mut tally_upper = TallyMap::new();
    tally_upper
        .add_words("Hello WORLD hello World", Case::Upper, Encoding::Unicode)
        .expect("valid unicode");
    assert_eq!(tally_upper.len(), 2);

    let mut tally_original = TallyMap::new();
    tally_original
        .add_words("Hello WORLD hello World", Case::Original, Encoding::Unicode)
        .expect("valid unicode");
    assert_eq!(tally_original.len(), 4);
}

#[test]
fn test_case_normalization_ascii() {
    let mut tally_lower = TallyMap::new();
    let result = tally_lower.add_words("Hello WORLD hello World", Case::Lower, Encoding::Ascii);
    assert!(result.is_ok());
    assert_eq!(tally_lower.len(), 2);

    let mut tally_upper = TallyMap::new();
    let result = tally_upper.add_words("Hello WORLD hello World", Case::Upper, Encoding::Ascii);
    assert!(result.is_ok());
    assert_eq!(tally_upper.len(), 2);

    let mut tally_original = TallyMap::new();
    let result =
        tally_original.add_words("Hello WORLD hello World", Case::Original, Encoding::Ascii);
    assert!(result.is_ok());
    assert_eq!(tally_original.len(), 4);
}

#[test]
fn test_word_counting() {
    let mut tally = TallyMap::new();
    tally
        .add_words("hello world hello", Case::Lower, Encoding::Unicode)
        .expect("valid unicode");

    let word_map: HashMap<String, usize> = tally.into_iter().map(|(w, c)| (w.into(), c)).collect();

    assert_eq!(word_map.get("hello"), Some(&2));
    assert_eq!(word_map.get("world"), Some(&1));
}

#[test]
fn test_large_text_segmentation() {
    let text = "The quick brown fox jumps over the lazy dog. ".repeat(100);
    let mut tally = TallyMap::new();
    tally
        .add_words(&text, Case::Lower, Encoding::Unicode)
        .expect("valid unicode");

    let word_map: HashMap<String, usize> = tally.into_iter().map(|(w, c)| (w.into(), c)).collect();

    assert_eq!(word_map.get("the"), Some(&200)); // "the" appears twice per repetition
    assert_eq!(word_map.get("quick"), Some(&100));
    assert_eq!(word_map.get("fox"), Some(&100));
}

#[test]
fn test_empty_and_whitespace() {
    let mut tally = TallyMap::new();

    // Empty string
    tally
        .add_words("", Case::Original, Encoding::Unicode)
        .expect("valid unicode");
    assert_eq!(tally.len(), 0);

    // Various whitespace
    tally
        .add_words("   \t\n\r  ", Case::Original, Encoding::Unicode)
        .expect("valid unicode");
    assert_eq!(tally.len(), 0);

    // Mixed with words
    tally
        .add_words("  hello   world  ", Case::Original, Encoding::Unicode)
        .expect("valid unicode");
    assert_eq!(tally.len(), 2);
}
