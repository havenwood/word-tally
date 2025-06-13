//! Tests for case normalization functionality.

use std::borrow::Cow;

use word_tally::options::case::Case;

#[test]
fn test_normalize_unicode_original_case() {
    let case = Case::Original;
    let test_cases = [
        ("narrow", "narrow"),
        ("FELLOW", "FELLOW"),
        ("CiRcUiT", "CiRcUiT"),
        ("", ""),
        ("123!@#", "123!@#"),
    ];

    for (input, expected) in test_cases {
        assert_eq!(case.normalize_unicode(input), Cow::Borrowed(expected));
    }
}

#[test]
fn test_normalize_unicode_lower_case_already_lower() {
    let case = Case::Lower;
    let test_cases = [
        "narrow",
        "fellow",
        "123",
        "",
        "!@#$%",
        "narrow123",
        "narrow_fellow",
    ];

    for input in test_cases {
        assert_eq!(case.normalize_unicode(input), Cow::Borrowed(input));
    }
}

#[test]
fn test_normalize_unicode_lower_case_needs_conversion() {
    let case = Case::Lower;
    let test_cases = [
        ("NARROW", "narrow"),
        ("NaRrOw", "narrow"),
        ("Fellow", "fellow"),
        ("CIRCUIT123", "circuit123"),
        ("ZERO_AT_BONE", "zero_at_bone"),
    ];

    for (input, expected) in test_cases {
        assert_eq!(
            case.normalize_unicode(input),
            Cow::Owned::<str>(expected.to_string())
        );
    }
}

#[test]
fn test_normalize_unicode_upper_case_already_upper() {
    let case = Case::Upper;
    let test_cases = [
        "NARROW",
        "FELLOW",
        "123",
        "",
        "!@#$%",
        "NARROW123",
        "NARROW_FELLOW",
    ];

    for input in test_cases {
        assert_eq!(case.normalize_unicode(input), Cow::Borrowed(input));
    }
}

#[test]
fn test_normalize_unicode_upper_case_needs_conversion() {
    let case = Case::Upper;
    let test_cases = [
        ("narrow", "NARROW"),
        ("NaRrOw", "NARROW"),
        ("Fellow", "FELLOW"),
        ("circuit123", "CIRCUIT123"),
        ("zero_at_bone", "ZERO_AT_BONE"),
    ];

    for (input, expected) in test_cases {
        assert_eq!(
            case.normalize_unicode(input),
            Cow::Owned::<str>(expected.to_string())
        );
    }
}

#[test]
fn test_normalize_ascii_lower_case() {
    let case = Case::Lower;

    // Already lowercase - borrowed
    let borrowed_cases = ["narrow", "fellow123", "", "!@#$%", "narrow_fellow"];
    for input in borrowed_cases {
        assert_eq!(case.normalize_ascii(input), Cow::Borrowed(input));
    }

    // Needs conversion - owned
    let owned_cases = [
        ("NARROW", "narrow"),
        ("NaRrOw", "narrow"),
        ("ZERO123", "zero123"),
    ];
    for (input, expected) in owned_cases {
        assert_eq!(
            case.normalize_ascii(input),
            Cow::Owned::<str>(expected.to_string())
        );
    }
}

#[test]
fn test_normalize_ascii_upper_case() {
    let case = Case::Upper;

    // Already uppercase - borrowed
    let borrowed_cases = ["NARROW", "FELLOW123", "", "!@#$%", "NARROW_FELLOW"];
    for input in borrowed_cases {
        assert_eq!(case.normalize_ascii(input), Cow::Borrowed(input));
    }

    // Needs conversion - owned
    let owned_cases = [
        ("narrow", "NARROW"),
        ("NaRrOw", "NARROW"),
        ("zero123", "ZERO123"),
    ];
    for (input, expected) in owned_cases {
        assert_eq!(
            case.normalize_ascii(input),
            Cow::Owned::<str>(expected.to_string())
        );
    }
}

#[test]
fn test_normalize_ascii_original_case() {
    let case = Case::Original;
    let test_cases = ["narrow", "NARROW", "NaRrOw", "", "123!@#"];

    for input in test_cases {
        assert_eq!(case.normalize_ascii(input), Cow::Borrowed(input));
    }
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
    let no_case_strings = ["12345", "!@#$%^&*()", "   "];

    for case in [Case::Lower, Case::Upper] {
        for input in &no_case_strings {
            assert_eq!(case.normalize_unicode(input), Cow::Borrowed(*input));
        }
    }
}

#[test]
fn test_normalize_ascii_edge_cases() {
    let no_case_strings = ["12345", "!@#$%^&*()", "   "];

    for case in [Case::Lower, Case::Upper] {
        for input in &no_case_strings {
            assert_eq!(case.normalize_ascii(input), Cow::Borrowed(*input));
        }
    }
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
