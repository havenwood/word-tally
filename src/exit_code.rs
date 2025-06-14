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
    Usage = 64,
    /// Data format error
    Data = 65,
    /// Cannot open input
    NoInput = 66,
    /// Internal software error
    Software = 70,
    /// Cannot create output
    CannotCreate = 73,
    /// I/O error
    Io = 74,
    /// Permission denied
    NoPermission = 77,
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

impl From<&WordTallyError> for ExitCode {
    fn from(err: &WordTallyError) -> Self {
        match err {
            WordTallyError::Usage(_)
            | WordTallyError::MmapStdin
            | WordTallyError::BytesWithPath
            | WordTallyError::BytesInputRequired
            | WordTallyError::Config(_) => Self::Usage,
            WordTallyError::Utf8 { .. }
            | WordTallyError::Pattern { .. }
            | WordTallyError::JsonSerialization(_)
            | WordTallyError::CsvSerialization(_)
            | WordTallyError::ChunkCountExceeded { .. }
            | WordTallyError::BatchSizeExceeded { .. }
            | WordTallyError::NonAsciiInAsciiMode { .. } => Self::Data,
            WordTallyError::MutexPoisoned => Self::Software,
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
