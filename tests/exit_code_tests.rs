use anyhow::{Error, anyhow};
use std::io;
use word_tally::exit_code::ExitCode;

fn create_io_error(kind: io::ErrorKind) -> Error {
    io::Error::new(kind, "test I/O error").into()
}

#[test]
fn test_io_error_not_found() {
    let err = create_io_error(io::ErrorKind::NotFound);
    assert_eq!(ExitCode::from_error(&err), ExitCode::NoInput);
}

#[test]
fn test_io_error_permission_denied() {
    let err = create_io_error(io::ErrorKind::PermissionDenied);
    assert_eq!(ExitCode::from_error(&err), ExitCode::NoPermission);
}

#[test]
fn test_io_error_already_exists() {
    let err = create_io_error(io::ErrorKind::AlreadyExists);
    assert_eq!(ExitCode::from_error(&err), ExitCode::CannotCreate);
}

#[test]
fn test_io_error_other() {
    let err = create_io_error(io::ErrorKind::ConnectionRefused);
    assert_eq!(ExitCode::from_error(&err), ExitCode::IoError);
}

#[test]
fn test_utf8_error() {
    // Invalid UTF-8 sequence
    let bytes = vec![0xFF, 0xFE, 0xFD];
    let utf8_error = std::str::from_utf8(&bytes).unwrap_err();
    let err: Error = utf8_error.into();
    assert_eq!(ExitCode::from_error(&err), ExitCode::DataError);
}

#[test]
fn test_from_utf8_error() {
    // Invalid UTF-8 sequence
    let bytes = vec![0xFF, 0xFE, 0xFD];
    let from_utf8_error = String::from_utf8(bytes).unwrap_err();
    let err: Error = from_utf8_error.into();
    assert_eq!(ExitCode::from_error(&err), ExitCode::DataError);
}

#[test]
// Intentionally invalid regex to test error handling
#[allow(clippy::invalid_regex)]
fn test_regex_error() {
    let regex_err = regex::Regex::new(r"[a-").unwrap_err();
    let err: Error = regex_err.into();
    assert_eq!(ExitCode::from_error(&err), ExitCode::DataError);
}

#[test]
fn test_json_error() {
    let invalid_json = "{invalid}";
    let json_err = serde_json::from_str::<serde_json::Value>(invalid_json).unwrap_err();
    let err: Error = json_err.into();
    assert_eq!(ExitCode::from_error(&err), ExitCode::DataError);
}

#[test]
fn test_csv_error() {
    use csv::ReaderBuilder;
    // Create a CSV error by having a mismatched quote
    let data = "a,b,c\n\"invalid";
    let mut reader = ReaderBuilder::new()
        .has_headers(false)
        .from_reader(data.as_bytes());
    // Skip the first valid record and get to the invalid one
    drop(
        reader
            .records()
            .next()
            .expect("process test")
            .expect("parse CSV record"),
    );
    let csv_err = reader.records().next().expect("process test").unwrap_err();
    let err: Error = csv_err.into();
    assert_eq!(ExitCode::from_error(&err), ExitCode::DataError);
}

#[test]
fn test_unescaper_error() {
    // Closing parenthesis without matching opening
    let unescaper_err = unescaper::unescape(r"\)").unwrap_err();
    let err: Error = unescaper_err.into();
    assert_eq!(ExitCode::from_error(&err), ExitCode::DataError);
}

#[test]
fn test_clap_help() {
    let clap_err = clap::Command::new("test")
        .disable_help_flag(true) // Disable the automatic help flag
        .arg(
            clap::Arg::new("help")
                .long("help")
                .action(clap::ArgAction::Help),
        )
        .try_get_matches_from(vec!["test", "--help"])
        .unwrap_err();
    let err: Error = anyhow!(clap_err);
    assert_eq!(ExitCode::from_error(&err), ExitCode::Success);
}

#[test]
fn test_clap_version() {
    let clap_err = clap::Command::new("test")
        .version("1.0")
        .try_get_matches_from(vec!["test", "--version"])
        .unwrap_err();
    let err: Error = anyhow!(clap_err);
    assert_eq!(ExitCode::from_error(&err), ExitCode::Success);
}

#[test]
fn test_generic_error() {
    let err: Error = anyhow!("generic error");
    assert_eq!(ExitCode::from_error(&err), ExitCode::Failure);
}

#[test]
fn test_exit_code_i32_conversion() {
    assert_eq!(ExitCode::Success.code(), 0);
    assert_eq!(ExitCode::Failure.code(), 1);
    assert_eq!(ExitCode::Usage.code(), 64);
    assert_eq!(ExitCode::DataError.code(), 65);
    assert_eq!(ExitCode::NoInput.code(), 66);
    assert_eq!(ExitCode::CannotCreate.code(), 73);
    assert_eq!(ExitCode::IoError.code(), 74);
    assert_eq!(ExitCode::NoPermission.code(), 77);
}

#[test]
fn test_from_trait() {
    let code: i32 = ExitCode::Success.into();
    assert_eq!(code, 0);

    let code: i32 = ExitCode::Usage.into();
    assert_eq!(code, 64);
}
