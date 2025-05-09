use anyhow::{Error, anyhow};
use std::io;
use word_tally::errors::{self, exit_code};

fn create_io_error(kind: io::ErrorKind) -> Error {
    io::Error::new(kind, "test I/O error").into()
}

#[test]
fn test_io_error_not_found() {
    let err = create_io_error(io::ErrorKind::NotFound);
    assert_eq!(errors::exit_code(&err), exit_code::NOINPUT);
}

#[test]
fn test_io_error_permission_denied() {
    let err = create_io_error(io::ErrorKind::PermissionDenied);
    assert_eq!(errors::exit_code(&err), exit_code::NOPERM);
}

#[test]
fn test_io_error_already_exists() {
    let err = create_io_error(io::ErrorKind::AlreadyExists);
    assert_eq!(errors::exit_code(&err), exit_code::CANTCREAT);
}

#[test]
fn test_io_error_other() {
    let err = create_io_error(io::ErrorKind::ConnectionRefused);
    assert_eq!(errors::exit_code(&err), exit_code::IOERR);
}

#[test]
fn test_utf8_error() {
    // Invalid UTF-8 sequence
    let bytes = vec![0xFF, 0xFE, 0xFD];
    let utf8_error = std::str::from_utf8(&bytes).unwrap_err();
    let err: Error = utf8_error.into();
    assert_eq!(errors::exit_code(&err), exit_code::DATAERR);
}

#[test]
fn test_from_utf8_error() {
    // Invalid UTF-8 sequence
    let bytes = vec![0xFF, 0xFE, 0xFD];
    let from_utf8_error = String::from_utf8(bytes).unwrap_err();
    let err: Error = from_utf8_error.into();
    assert_eq!(errors::exit_code(&err), exit_code::DATAERR);
}

#[test]
#[allow(clippy::invalid_regex)]
fn test_regex_error() {
    let regex_error = regex::Regex::new("(invalid regex").unwrap_err();
    let err: Error = regex_error.into();
    assert_eq!(errors::exit_code(&err), exit_code::DATAERR);
}

#[test]
fn test_serde_json_error() {
    let json_error = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
    let err: Error = json_error.into();
    assert_eq!(errors::exit_code(&err), exit_code::DATAERR);
}

#[test]
fn test_csv_error() {
    let mut reader = csv::ReaderBuilder::new().from_reader(&b"a,b\n\"unclosed quote"[..]);

    let mut record = csv::StringRecord::new();
    let csv_error = reader.read_record(&mut record).unwrap_err();

    let err: Error = csv_error.into();
    assert_eq!(errors::exit_code(&err), exit_code::DATAERR);
}

#[test]
fn test_unescaper_error() {
    let unescaper_error = unescaper::unescape("\\u{invalid}").unwrap_err();
    let err: Error = unescaper_error.into();
    assert_eq!(errors::exit_code(&err), exit_code::CONFIG);
}

#[test]
fn test_clap_error() {
    use clap::{Arg, Command};
    let cmd = Command::new("test").arg(Arg::new("arg").required(true));
    let clap_error = cmd.try_get_matches_from(vec!["test"]).unwrap_err();
    let err: Error = clap_error.into();
    assert_eq!(errors::exit_code(&err), exit_code::USAGE);
}

#[test]
fn test_clap_help_success() {
    use clap::Command;
    let cmd = Command::new("test");
    let matches = cmd.try_get_matches_from(vec!["test", "--help"]);
    assert!(matches.is_err(), "Expected error for --help");
    let err: Error = matches.unwrap_err().into();
    assert_eq!(errors::exit_code(&err), exit_code::SUCCESS);
}

#[test]
fn test_nested_error() {
    let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
    let nested_err = anyhow::anyhow!(io_error).context("Failed to process");
    assert_eq!(errors::exit_code(&nested_err), exit_code::NOINPUT);
}

#[test]
fn test_unknown_error() {
    let err = anyhow!("unknown error type");
    assert_eq!(errors::exit_code(&err), exit_code::FAILURE);
}
