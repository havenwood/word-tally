//! Error exit codes for word-tally
//!
//! Follows Unix sysexits.h convention for exit code numbers.

use anyhow::Error;
use clap::error::ErrorKind as ClapErrorKind;
use std::io;

pub mod exit_code {
    pub const SUCCESS: i32 = 0;
    pub const FAILURE: i32 = 1;
    pub const USAGE: i32 = 64;
    pub const DATAERR: i32 = 65;
    pub const NOINPUT: i32 = 66;
    pub const CANTCREAT: i32 = 73;
    pub const IOERR: i32 = 74;
    pub const NOPERM: i32 = 77;
    pub const CONFIG: i32 = 78;
}

/// Helper function to check if an error is one of multiple types
fn is_error_any_of<T: std::fmt::Debug + std::fmt::Display + Send + Sync + 'static>(
    err: &Error,
) -> bool {
    err.downcast_ref::<T>().is_some()
}

pub fn exit_code(err: &Error) -> i32 {
    if let Some(clap_err) = err.downcast_ref::<clap::Error>() {
        return match clap_err.kind() {
            ClapErrorKind::DisplayHelp | ClapErrorKind::DisplayVersion => exit_code::SUCCESS,
            _ => exit_code::USAGE,
        };
    }

    if let Some(io_err) = err.downcast_ref::<io::Error>() {
        return match io_err.kind() {
            io::ErrorKind::NotFound => exit_code::NOINPUT,
            io::ErrorKind::PermissionDenied => exit_code::NOPERM,
            io::ErrorKind::AlreadyExists => exit_code::CANTCREAT,
            _ => exit_code::IOERR,
        };
    }

    // Data errors
    if is_error_any_of::<regex::Error>(err)
        || is_error_any_of::<serde_json::Error>(err)
        || is_error_any_of::<csv::Error>(err)
        || is_error_any_of::<std::str::Utf8Error>(err)
        || is_error_any_of::<std::string::FromUtf8Error>(err)
    {
        return exit_code::DATAERR;
    }

    // Configuration errors
    if is_error_any_of::<unescaper::Error>(err) {
        return exit_code::CONFIG;
    }

    exit_code::FAILURE
}
