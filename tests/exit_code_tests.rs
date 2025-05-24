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
    assert_eq!(ExitCode::from_error(&err), ExitCode::Io);
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
fn test_exit_code_values() {
    assert_eq!(ExitCode::Success as i32, 0);
    assert_eq!(ExitCode::Failure as i32, 1);
    assert_eq!(ExitCode::Usage as i32, 64);
    assert_eq!(ExitCode::Data as i32, 65);
    assert_eq!(ExitCode::NoInput as i32, 66);
    assert_eq!(ExitCode::CannotCreate as i32, 73);
    assert_eq!(ExitCode::Io as i32, 74);
    assert_eq!(ExitCode::NoPermission as i32, 77);
}
