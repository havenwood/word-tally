//! Error exit codes for word-tally
//!
//! Follows Unix sysexits.h convention for exit code numbers.

use anyhow::Error;
use clap::error::ErrorKind as ClapErrorKind;
use std::fmt::{Debug, Display};
use std::io;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitCode {
    Success = 0,
    Failure = 1,
    Usage = 64,
    DataError = 65,
    NoInput = 66,
    CannotCreate = 73,
    IoError = 74,
    NoPermission = 77,
}

impl ExitCode {
    /// Converts an error to an appropriate exit code
    pub fn from_error(err: &Error) -> Self {
        if let Some(clap_err) = err.downcast_ref::<clap::Error>() {
            return match clap_err.kind() {
                // Successful `--help` or `--version` display
                ClapErrorKind::DisplayHelp | ClapErrorKind::DisplayVersion => Self::Success,
                // Clap usage errors
                _ => Self::Usage,
            };
        }

        if let Some(io_err) = err.downcast_ref::<io::Error>() {
            return match io_err.kind() {
                // No input errors
                io::ErrorKind::NotFound => Self::NoInput,
                // Permission errors
                io::ErrorKind::PermissionDenied => Self::NoPermission,
                // Inability to create errors
                io::ErrorKind::AlreadyExists => Self::CannotCreate,
                // Other I/O errors
                _ => Self::IoError,
            };
        }

        // Data errors
        if is_error_any_of::<regex::Error>(err)
            || is_error_any_of::<serde_json::Error>(err)
            || is_error_any_of::<csv::Error>(err)
            || is_error_any_of::<std::str::Utf8Error>(err)
            || is_error_any_of::<std::string::FromUtf8Error>(err)
            || is_error_any_of::<unescaper::Error>(err)
        {
            return Self::DataError;
        }

        // Any other errors
        Self::Failure
    }

    /// Returns the numeric exit code value
    pub const fn code(self) -> i32 {
        self as i32
    }
}

impl From<ExitCode> for i32 {
    fn from(code: ExitCode) -> Self {
        code.code()
    }
}

/// Helper function to check if an error is one of multiple types
fn is_error_any_of<T: Debug + Display + Send + Sync + 'static>(err: &Error) -> bool {
    err.downcast_ref::<T>().is_some()
}
