//! Exit codes for word-tally following Unix sysexits.h convention

use crate::error::Error;
use clap::error::ErrorKind as ClapErrorKind;
use std::io;

/// Exit codes following Unix sysexits.h convention
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ExitCode {
    /// Successful termination
    Success = 0,
    /// General failure
    Failure = 1,
    /// Command line usage error
    Usage = 64,
    /// Data format error
    Data = 65,
    /// Cannot open input
    NoInput = 66,
    /// Cannot create output
    CannotCreate = 73,
    /// I/O error
    Io = 74,
    /// Permission denied
    NoPermission = 77,
}

impl ExitCode {
    /// Converts an error to an appropriate exit code.
    #[must_use]
    pub fn from_error(err: &anyhow::Error) -> Self {
        Self::from(err)
    }
}

impl From<&clap::Error> for ExitCode {
    fn from(err: &clap::Error) -> Self {
        match err.kind() {
            // Successful `--help` or `--version` display
            ClapErrorKind::DisplayHelp | ClapErrorKind::DisplayVersion => Self::Success,
            // Clap usage errors
            _ => Self::Usage,
        }
    }
}

impl From<&Error> for ExitCode {
    fn from(err: &Error) -> Self {
        match err {
            Error::Usage(_)
            | Error::MmapStdin
            | Error::BytesWithPath
            | Error::BytesInputRequired
            | Error::Config(_) => Self::Usage,
            Error::Utf8 { .. }
            | Error::Pattern { .. }
            | Error::JsonSerialization(_)
            | Error::CsvSerialization(_)
            | Error::ChunkCountExceeded { .. }
            | Error::BatchSizeExceeded { .. }
            | Error::NonAsciiInAsciiMode { .. } => Self::Data,
            Error::Io { source, .. } => Self::from(source),
        }
    }
}

impl From<&io::Error> for ExitCode {
    fn from(err: &io::Error) -> Self {
        match err.kind() {
            io::ErrorKind::NotFound => Self::NoInput,
            io::ErrorKind::PermissionDenied => Self::NoPermission,
            io::ErrorKind::AlreadyExists => Self::CannotCreate,
            _ => Self::Io,
        }
    }
}

impl From<&anyhow::Error> for ExitCode {
    fn from(err: &anyhow::Error) -> Self {
        if let Some(clap_err) = err.downcast_ref::<clap::Error>() {
            return Self::from(clap_err);
        }

        if let Some(wt_err) = err.downcast_ref::<Error>() {
            return Self::from(wt_err);
        }

        if let Some(io_err) = err.downcast_ref::<io::Error>() {
            return Self::from(io_err);
        }

        Self::Failure
    }
}

impl From<ExitCode> for i32 {
    fn from(code: ExitCode) -> Self {
        code as Self
    }
}
