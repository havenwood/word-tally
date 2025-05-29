use word_tally::{Case, Io, MinChars, Sort};

#[test]
fn test_format_enum_as_string() {
    // Test direct string formatting using Display trait
    let case = Case::Lower;
    assert_eq!(case.to_string(), "lower");

    let sort = Sort::Desc;
    assert_eq!(sort.to_string(), "desc");

    let io = Io::ParallelStream;
    assert_eq!(io.to_string(), "parallel-stream");
}

#[test]
fn test_format_option() {
    // Test formatting of options
    let some_min_chars: Option<MinChars> = Some(3);
    assert_eq!(
        some_min_chars
            .as_ref()
            .map(std::string::ToString::to_string),
        Some("3".to_string())
    );

    let none_min_chars: Option<MinChars> = None;
    assert_eq!(
        none_min_chars
            .as_ref()
            .map(std::string::ToString::to_string),
        None
    );
}
