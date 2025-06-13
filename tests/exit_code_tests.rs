//! Tests for exit code handling.

use std::io;

use anyhow::{Error, anyhow};
use word_tally::{WordTallyError, exit_code::ExitCode};

fn create_io_error(kind: io::ErrorKind) -> Error {
    io::Error::new(kind, "test I/O error").into()
}

#[test]
fn test_io_error_not_found() {
    let err = create_io_error(io::ErrorKind::NotFound);
    assert_eq!(ExitCode::from(&err), ExitCode::NoInput);
}

#[test]
fn test_io_error_permission_denied() {
    let err = create_io_error(io::ErrorKind::PermissionDenied);
    assert_eq!(ExitCode::from(&err), ExitCode::NoPermission);
}

#[test]
fn test_io_error_already_exists() {
    let err = create_io_error(io::ErrorKind::AlreadyExists);
    assert_eq!(ExitCode::from(&err), ExitCode::CannotCreate);
}

#[test]
fn test_io_error_other() {
    let err = create_io_error(io::ErrorKind::ConnectionRefused);
    assert_eq!(ExitCode::from(&err), ExitCode::Io);
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
        .expect_err("--help flag should produce an error");
    let err: Error = anyhow!(clap_err);
    assert_eq!(ExitCode::from(&err), ExitCode::Success);
}

#[test]
fn test_clap_version() {
    let clap_err = clap::Command::new("test")
        .version("1.0")
        .try_get_matches_from(vec!["test", "--version"])
        .expect_err("--version flag should produce an error");
    let err: Error = anyhow!(clap_err);
    assert_eq!(ExitCode::from(&err), ExitCode::Success);
}

#[test]
fn test_generic_error() {
    let err: Error = anyhow!("generic error");
    assert_eq!(ExitCode::from(&err), ExitCode::Failure);
}

#[test]
fn test_exit_code_values() {
    assert_eq!(ExitCode::Success as i32, 0);
    assert_eq!(ExitCode::Failure as i32, 1);
    assert_eq!(ExitCode::Usage as i32, 64);
    assert_eq!(ExitCode::Data as i32, 65);
    assert_eq!(ExitCode::NoInput as i32, 66);
    assert_eq!(ExitCode::Software as i32, 70);
    assert_eq!(ExitCode::CannotCreate as i32, 73);
    assert_eq!(ExitCode::Io as i32, 74);
    assert_eq!(ExitCode::NoPermission as i32, 77);
}

#[test]
fn test_exit_code_from_io_error_trait() {
    let not_found = io::Error::new(io::ErrorKind::NotFound, "test");
    assert_eq!(ExitCode::from(&not_found), ExitCode::NoInput);

    let permission_denied = io::Error::new(io::ErrorKind::PermissionDenied, "test");
    assert_eq!(ExitCode::from(&permission_denied), ExitCode::NoPermission);

    let already_exists = io::Error::new(io::ErrorKind::AlreadyExists, "test");
    assert_eq!(ExitCode::from(&already_exists), ExitCode::CannotCreate);

    let other = io::Error::other("test");
    assert_eq!(ExitCode::from(&other), ExitCode::Io);
}

#[test]
fn test_exit_code_from_clap_error_trait() {
    let help_err = clap::Command::new("test")
        .disable_help_flag(true)
        .arg(
            clap::Arg::new("help")
                .long("help")
                .action(clap::ArgAction::Help),
        )
        .try_get_matches_from(vec!["test", "--help"])
        .expect_err("--help flag should produce an error");
    assert_eq!(ExitCode::from(&help_err), ExitCode::Success);

    let version_err = clap::Command::new("test")
        .version("1.0")
        .try_get_matches_from(vec!["test", "--version"])
        .expect_err("--version flag should produce an error");
    assert_eq!(ExitCode::from(&version_err), ExitCode::Success);
}

#[test]
fn test_exit_code_from_anyhow_error_trait() {
    let io_err = io::Error::new(io::ErrorKind::NotFound, "test");
    let anyhow_err: Error = io_err.into();
    assert_eq!(ExitCode::from(&anyhow_err), ExitCode::NoInput);

    let generic_err: Error = anyhow!("generic error");
    assert_eq!(ExitCode::from(&generic_err), ExitCode::Failure);

    // Test that `from_error` still works and delegates to `From` trait
    assert_eq!(
        ExitCode::from_error(&generic_err),
        ExitCode::from(&generic_err)
    );
}

#[test]
fn test_mutex_poisoned_exit_code() {
    let mutex_err = WordTallyError::MutexPoisoned;
    let anyhow_err: Error = mutex_err.into();
    assert_eq!(ExitCode::from(&anyhow_err), ExitCode::Software);
}
