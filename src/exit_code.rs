//! Error exit codes for word-tally
//!
//! Follows Unix sysexits.h convention for exit code numbers.

use anyhow::Error;
use clap::error::ErrorKind as ClapErrorKind;
use std::fmt::{Debug, Display};
use std::io;

pub const SUCCESS: i32 = 0;
pub const FAILURE: i32 = 1;
pub const USAGE: i32 = 64;
pub const DATA_ERROR: i32 = 65;
pub const NO_INPUT: i32 = 66;
pub const CANNOT_CREATE: i32 = 73;
pub const IO_ERROR: i32 = 74;
pub const NO_PERMISSION: i32 = 77;

/// Converts an error to an appropriate exit code.
#[must_use]
pub fn from_error(err: &Error) -> i32 {
    // Check Clap errors (during argument parsing)
    if let Some(clap_err) = err.downcast_ref::<clap::Error>() {
        return match clap_err.kind() {
            // Successful `--help` or `--version` display
            ClapErrorKind::DisplayHelp | ClapErrorKind::DisplayVersion => SUCCESS,
            // Clap usage errors
            _ => USAGE,
        };
    }

    // Check I/O errors (when opening files)
    if let Some(io_err) = err.downcast_ref::<io::Error>() {
        return match io_err.kind() {
            // No input errors
            io::ErrorKind::NotFound => NO_INPUT,
            // Permission errors
            io::ErrorKind::PermissionDenied => NO_PERMISSION,
            // Inability to create errors
            io::ErrorKind::AlreadyExists => CANNOT_CREATE,
            // Other I/O errors
            _ => IO_ERROR,
        };
    }

    // Check data errors (during processing)
    if is_error_type::<regex::Error>(err)
        || is_error_type::<serde_json::Error>(err)
        || is_error_type::<csv::Error>(err)
        || is_error_type::<std::str::Utf8Error>(err)
        || is_error_type::<std::string::FromUtf8Error>(err)
        || is_error_type::<unescaper::Error>(err)
    {
        return DATA_ERROR;
    }

    // Any other errors
    FAILURE
}

/// Helper function to check if an error is of a specific type.
fn is_error_type<E: Display + Debug + Send + Sync + 'static>(err: &Error) -> bool {
    err.downcast_ref::<E>().is_some()
}
