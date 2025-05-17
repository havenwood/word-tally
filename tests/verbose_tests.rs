use word_tally::{Case, Io, MinChars, Processing, Sort};

#[test]
fn test_format_enum_as_string() {
    // Test direct string formatting using Display trait
    let case = Case::Lower;
    assert_eq!(case.to_string(), "lower");

    let sort = Sort::Desc;
    assert_eq!(sort.to_string(), "desc");

    let processing = Processing::Sequential;
    assert_eq!(processing.to_string(), "sequential");

    let io = Io::Streamed;
    assert_eq!(io.to_string(), "streamed");
}

#[test]
fn test_format_option() {
    // Test formatting of options
    let some_min_chars: Option<MinChars> = Some(3);
    assert_eq!(
        some_min_chars.as_ref().map(|v| v.to_string()),
        Some("3".to_string())
    );

    let none_min_chars: Option<MinChars> = None;
    assert_eq!(none_min_chars.as_ref().map(|v| v.to_string()), None);
}
