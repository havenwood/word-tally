//! Tests for case normalization functionality.

use std::borrow::Cow;
use word_tally::options::case::Case;

#[test]
fn test_normalize_unicode_original_case() {
    let case = Case::Original;

    assert_eq!(case.normalize_unicode("narrow"), Cow::Borrowed("narrow"));
    assert_eq!(case.normalize_unicode("FELLOW"), Cow::Borrowed("FELLOW"));
    assert_eq!(case.normalize_unicode("CiRcUiT"), Cow::Borrowed("CiRcUiT"));
    assert_eq!(case.normalize_unicode(""), Cow::Borrowed(""));
    assert_eq!(case.normalize_unicode("123!@#"), Cow::Borrowed("123!@#"));
}

#[test]
fn test_normalize_unicode_lower_case_already_lower() {
    let case = Case::Lower;

    // Already lowercase - should return borrowed
    assert_eq!(case.normalize_unicode("narrow"), Cow::Borrowed("narrow"));
    assert_eq!(case.normalize_unicode("fellow"), Cow::Borrowed("fellow"));
    assert_eq!(case.normalize_unicode("123"), Cow::Borrowed("123"));
    assert_eq!(case.normalize_unicode(""), Cow::Borrowed(""));
    assert_eq!(case.normalize_unicode("!@#$%"), Cow::Borrowed("!@#$%"));
    assert_eq!(
        case.normalize_unicode("narrow123"),
        Cow::Borrowed("narrow123")
    );
    assert_eq!(
        case.normalize_unicode("narrow_fellow"),
        Cow::Borrowed("narrow_fellow")
    );
}

#[test]
fn test_normalize_unicode_lower_case_needs_conversion() {
    let case = Case::Lower;

    // Has uppercase - should return owned
    assert_eq!(
        case.normalize_unicode("NARROW"),
        Cow::Owned::<str>("narrow".to_string())
    );
    assert_eq!(
        case.normalize_unicode("NaRrOw"),
        Cow::Owned::<str>("narrow".to_string())
    );
    assert_eq!(
        case.normalize_unicode("Fellow"),
        Cow::Owned::<str>("fellow".to_string())
    );
    assert_eq!(
        case.normalize_unicode("CIRCUIT123"),
        Cow::Owned::<str>("circuit123".to_string())
    );
    assert_eq!(
        case.normalize_unicode("ZERO_AT_BONE"),
        Cow::Owned::<str>("zero_at_bone".to_string())
    );
}

#[test]
fn test_normalize_unicode_upper_case_already_upper() {
    let case = Case::Upper;

    // Already uppercase - should return borrowed
    assert_eq!(case.normalize_unicode("NARROW"), Cow::Borrowed("NARROW"));
    assert_eq!(case.normalize_unicode("FELLOW"), Cow::Borrowed("FELLOW"));
    assert_eq!(case.normalize_unicode("123"), Cow::Borrowed("123"));
    assert_eq!(case.normalize_unicode(""), Cow::Borrowed(""));
    assert_eq!(case.normalize_unicode("!@#$%"), Cow::Borrowed("!@#$%"));
    assert_eq!(
        case.normalize_unicode("NARROW123"),
        Cow::Borrowed("NARROW123")
    );
    assert_eq!(
        case.normalize_unicode("NARROW_FELLOW"),
        Cow::Borrowed("NARROW_FELLOW")
    );
}

#[test]
fn test_normalize_unicode_upper_case_needs_conversion() {
    let case = Case::Upper;

    // Has lowercase - should return owned
    assert_eq!(
        case.normalize_unicode("narrow"),
        Cow::Owned::<str>("NARROW".to_string())
    );
    assert_eq!(
        case.normalize_unicode("NaRrOw"),
        Cow::Owned::<str>("NARROW".to_string())
    );
    assert_eq!(
        case.normalize_unicode("Fellow"),
        Cow::Owned::<str>("FELLOW".to_string())
    );
    assert_eq!(
        case.normalize_unicode("circuit123"),
        Cow::Owned::<str>("CIRCUIT123".to_string())
    );
    assert_eq!(
        case.normalize_unicode("zero_at_bone"),
        Cow::Owned::<str>("ZERO_AT_BONE".to_string())
    );
}

#[test]
fn test_normalize_ascii_lower_case() {
    let case = Case::Lower;

    // Already lowercase ASCII - should return borrowed
    assert_eq!(case.normalize_ascii("narrow"), Cow::Borrowed("narrow"));
    assert_eq!(
        case.normalize_ascii("fellow123"),
        Cow::Borrowed("fellow123")
    );
    assert_eq!(case.normalize_ascii(""), Cow::Borrowed(""));
    assert_eq!(case.normalize_ascii("!@#$%"), Cow::Borrowed("!@#$%"));
    assert_eq!(
        case.normalize_ascii("narrow_fellow"),
        Cow::Borrowed("narrow_fellow")
    );

    // Has uppercase ASCII - should return owned
    assert_eq!(
        case.normalize_ascii("NARROW"),
        Cow::Owned::<str>("narrow".to_string())
    );
    assert_eq!(
        case.normalize_ascii("NaRrOw"),
        Cow::Owned::<str>("narrow".to_string())
    );
    assert_eq!(
        case.normalize_ascii("ZERO123"),
        Cow::Owned::<str>("zero123".to_string())
    );
}

#[test]
fn test_normalize_ascii_upper_case() {
    let case = Case::Upper;

    // Already uppercase ASCII - should return borrowed
    assert_eq!(case.normalize_ascii("NARROW"), Cow::Borrowed("NARROW"));
    assert_eq!(
        case.normalize_ascii("FELLOW123"),
        Cow::Borrowed("FELLOW123")
    );
    assert_eq!(case.normalize_ascii(""), Cow::Borrowed(""));
    assert_eq!(case.normalize_ascii("!@#$%"), Cow::Borrowed("!@#$%"));
    assert_eq!(
        case.normalize_ascii("NARROW_FELLOW"),
        Cow::Borrowed("NARROW_FELLOW")
    );

    // Has lowercase ASCII - should return owned
    assert_eq!(
        case.normalize_ascii("narrow"),
        Cow::Owned::<str>("NARROW".to_string())
    );
    assert_eq!(
        case.normalize_ascii("NaRrOw"),
        Cow::Owned::<str>("NARROW".to_string())
    );
    assert_eq!(
        case.normalize_ascii("zero123"),
        Cow::Owned::<str>("ZERO123".to_string())
    );
}

#[test]
fn test_normalize_ascii_original_case() {
    let case = Case::Original;

    // Always returns borrowed for original case
    assert_eq!(case.normalize_ascii("narrow"), Cow::Borrowed("narrow"));
    assert_eq!(case.normalize_ascii("NARROW"), Cow::Borrowed("NARROW"));
    assert_eq!(case.normalize_ascii("NaRrOw"), Cow::Borrowed("NaRrOw"));
    assert_eq!(case.normalize_ascii(""), Cow::Borrowed(""));
    assert_eq!(case.normalize_ascii("123!@#"), Cow::Borrowed("123!@#"));
}

#[test]
fn test_normalize_unicode_unicode_characters() {
    // Test lowercase conversion with various Unicode characters
    let case = Case::Lower;
    assert_eq!(
        case.normalize_unicode("CAFÉ"),
        Cow::Owned::<str>("café".to_string())
    );
    assert_eq!(case.normalize_unicode("café"), Cow::Borrowed("café"));
    assert_eq!(
        case.normalize_unicode("NAÏVE"),
        Cow::Owned::<str>("naïve".to_string())
    );
    assert_eq!(
        case.normalize_unicode("ÜBUNG"),
        Cow::Owned::<str>("übung".to_string())
    );
    assert_eq!(
        case.normalize_unicode("МОСКВА"),
        Cow::Owned::<str>("москва".to_string())
    );
    assert_eq!(case.normalize_unicode("москва"), Cow::Borrowed("москва"));

    // Test uppercase conversion with various Unicode characters
    let case = Case::Upper;
    assert_eq!(
        case.normalize_unicode("café"),
        Cow::Owned::<str>("CAFÉ".to_string())
    );
    assert_eq!(case.normalize_unicode("CAFÉ"), Cow::Borrowed("CAFÉ"));
    assert_eq!(
        case.normalize_unicode("naïve"),
        Cow::Owned::<str>("NAÏVE".to_string())
    );
    assert_eq!(
        case.normalize_unicode("übung"),
        Cow::Owned::<str>("ÜBUNG".to_string())
    );
    assert_eq!(
        case.normalize_unicode("москва"),
        Cow::Owned::<str>("МОСКВА".to_string())
    );
    assert_eq!(case.normalize_unicode("МОСКВА"), Cow::Borrowed("МОСКВА"));
}

#[test]
fn test_normalize_ascii_with_non_ascii_chars() {
    // ASCII normalization with non-ASCII characters
    // The methods should still work but only affect ASCII characters
    let case = Case::Lower;
    assert_eq!(
        case.normalize_ascii("CAFé"),
        Cow::Owned::<str>("café".to_string())
    );
    assert_eq!(case.normalize_ascii("café"), Cow::Borrowed("café"));

    let case = Case::Upper;
    assert_eq!(
        case.normalize_ascii("cafÉ"),
        Cow::Owned::<str>("CAFÉ".to_string())
    );
    assert_eq!(case.normalize_ascii("CAFÉ"), Cow::Borrowed("CAFÉ"));
}

#[test]
fn test_normalize_unicode_edge_cases() {
    // Test with strings that have no case (numbers, symbols)
    let case = Case::Lower;
    assert_eq!(case.normalize_unicode("12345"), Cow::Borrowed("12345"));
    assert_eq!(
        case.normalize_unicode("!@#$%^&*()"),
        Cow::Borrowed("!@#$%^&*()")
    );
    assert_eq!(case.normalize_unicode("   "), Cow::Borrowed("   "));

    let case = Case::Upper;
    assert_eq!(case.normalize_unicode("12345"), Cow::Borrowed("12345"));
    assert_eq!(
        case.normalize_unicode("!@#$%^&*()"),
        Cow::Borrowed("!@#$%^&*()")
    );
    assert_eq!(case.normalize_unicode("   "), Cow::Borrowed("   "));
}

#[test]
fn test_normalize_ascii_edge_cases() {
    // Test with strings that have no case (numbers, symbols)
    let case = Case::Lower;
    assert_eq!(case.normalize_ascii("12345"), Cow::Borrowed("12345"));
    assert_eq!(
        case.normalize_ascii("!@#$%^&*()"),
        Cow::Borrowed("!@#$%^&*()")
    );
    assert_eq!(case.normalize_ascii("   "), Cow::Borrowed("   "));

    let case = Case::Upper;
    assert_eq!(case.normalize_ascii("12345"), Cow::Borrowed("12345"));
    assert_eq!(
        case.normalize_ascii("!@#$%^&*()"),
        Cow::Borrowed("!@#$%^&*()")
    );
    assert_eq!(case.normalize_ascii("   "), Cow::Borrowed("   "));
}

#[test]
fn test_normalize_unicode_special_case_mappings() {
    // Test special Unicode case mappings
    let case = Case::Lower;
    assert_eq!(
        case.normalize_unicode("İstanbul"), // Turkish capital I with dot
        Cow::Owned::<str>("i̇stanbul".to_string())
    );
    assert_eq!(
        case.normalize_unicode("ß"), // German eszett
        Cow::Borrowed("ß")           // Already lowercase
    );

    let case = Case::Upper;
    assert_eq!(
        case.normalize_unicode("ß"),         // German eszett
        Cow::Owned::<str>("SS".to_string())  // Uppercases to SS
    );
}

#[test]
fn test_normalize_mixed_content() {
    // Test strings with mixed content (letters, numbers, symbols)
    let case = Case::Lower;
    assert_eq!(
        case.normalize_unicode("Circuit123Rider!"),
        Cow::Owned::<str>("circuit123rider!".to_string())
    );
    assert_eq!(
        case.normalize_unicode("circuit123rider!"),
        Cow::Borrowed("circuit123rider!")
    );

    let case = Case::Upper;
    assert_eq!(
        case.normalize_unicode("Circuit123Rider!"),
        Cow::Owned::<str>("CIRCUIT123RIDER!".to_string())
    );
    assert_eq!(
        case.normalize_unicode("CIRCUIT123RIDER!"),
        Cow::Borrowed("CIRCUIT123RIDER!")
    );
}

#[test]
fn test_normalize_ascii_preserves_semantics() {
    // Test that ASCII normalization preserves the same semantics for ASCII strings
    let test_strings = [
        "narrow",
        "NARROW",
        "NaRrOw",
        "fellow123",
        "ZERO",
        "",
        "123",
        "!@#",
        "narrow_fellow",
        "ZERO_AT_BONE",
    ];

    for s in &test_strings {
        for case in [Case::Original, Case::Lower, Case::Upper] {
            let unicode_result = case.normalize_unicode(s);
            let ascii_result = case.normalize_ascii(s);

            // For ASCII strings, both methods should produce the same result
            assert_eq!(unicode_result.as_ref(), ascii_result.as_ref());
        }
    }
}

#[test]
fn test_case_display() {
    assert_eq!(Case::Lower.to_string(), "lower");
    assert_eq!(Case::Upper.to_string(), "upper");
    assert_eq!(Case::Original.to_string(), "original");
}

#[test]
fn test_case_default() {
    assert_eq!(Case::default(), Case::Original);
}
