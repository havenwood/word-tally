//! Exit codes following Unix sysexits.h conventions.

use std::{io, process};

use clap::error::ErrorKind as ClapErrorKind;

use crate::error::Error as WordTallyError;

/// Exit codes following Unix sysexits.h convention
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum ExitCode {
    /// Successful termination
    Success = 0,
    /// General failure
    Failure = 1,
    /// Command line usage error
    UsageError = 64,
    /// Data format error
    DataFormat = 65,
    /// Cannot open input
    InputNotFound = 66,
    /// Internal software error
    InternalError = 70,
    /// Cannot create output
    OutputFailed = 73,
    /// I/O error
    IoError = 74,
    /// Permission denied
    PermissionDenied = 77,
}

impl From<&io::Error> for ExitCode {
    fn from(err: &io::Error) -> Self {
        match err.kind() {
            io::ErrorKind::NotFound => Self::InputNotFound,
            io::ErrorKind::PermissionDenied => Self::PermissionDenied,
            io::ErrorKind::AlreadyExists => Self::OutputFailed,
            _ => Self::IoError,
        }
    }
}

impl From<&clap::Error> for ExitCode {
    fn from(err: &clap::Error) -> Self {
        match err.kind() {
            // Successful `--help` or `--version` display
            ClapErrorKind::DisplayHelp | ClapErrorKind::DisplayVersion => Self::Success,
            // Clap usage errors
            _ => Self::UsageError,
        }
    }
}

impl From<&WordTallyError> for ExitCode {
    fn from(err: &WordTallyError) -> Self {
        match err {
            WordTallyError::Usage(_)
            | WordTallyError::StdinInvalid
            | WordTallyError::PathInvalid
            | WordTallyError::BytesRequired
            | WordTallyError::Config(_) => Self::UsageError,
            WordTallyError::Utf8 { .. }
            | WordTallyError::Pattern { .. }
            | WordTallyError::Json(_)
            | WordTallyError::Csv(_)
            | WordTallyError::ChunkOverflow { .. }
            | WordTallyError::BatchOverflow { .. }
            | WordTallyError::NonAscii { .. } => Self::DataFormat,
            WordTallyError::MutexPoisoned => Self::InternalError,
            WordTallyError::Io { source, .. } => Self::from(source),
        }
    }
}

impl From<&anyhow::Error> for ExitCode {
    fn from(err: &anyhow::Error) -> Self {
        err.downcast_ref::<io::Error>()
            .map(Self::from)
            .or_else(|| err.downcast_ref::<clap::Error>().map(Self::from))
            .or_else(|| err.downcast_ref::<WordTallyError>().map(Self::from))
            .unwrap_or(Self::Failure)
    }
}

impl From<ExitCode> for process::ExitCode {
    fn from(code: ExitCode) -> Self {
        Self::from(code as u8)
    }
}
