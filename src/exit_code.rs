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
    /// Converts an error to an appropriate exit code.
    pub fn from_error(err: &Error) -> Self {
        // Check Clap errors (during argument parsing)
        if let Some(clap_err) = err.downcast_ref::<clap::Error>() {
            return match clap_err.kind() {
                // Successful `--help` or `--version` display
                ClapErrorKind::DisplayHelp | ClapErrorKind::DisplayVersion => Self::Success,
                // Clap usage errors
                _ => Self::Usage,
            };
        }

        // Check I/O errors (when opening files)
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

        // Check data errors (during processing)
        if Self::is_error_type::<regex::Error>(err)
            || Self::is_error_type::<serde_json::Error>(err)
            || Self::is_error_type::<csv::Error>(err)
            || Self::is_error_type::<std::str::Utf8Error>(err)
            || Self::is_error_type::<std::string::FromUtf8Error>(err)
            || Self::is_error_type::<unescaper::Error>(err)
        {
            return Self::DataError;
        }

        // Any other errors
        Self::Failure
    }

    /// Returns the numeric exit code value.
    pub const fn code(self) -> i32 {
        self as i32
    }

    /// Helper function to check if an error is of a specific type.
    fn is_error_type<E: Display + Debug + Send + Sync + 'static>(err: &Error) -> bool {
        err.downcast_ref::<E>().is_some()
    }
}

impl From<ExitCode> for i32 {
    fn from(code: ExitCode) -> Self {
        code.code()
    }
}
