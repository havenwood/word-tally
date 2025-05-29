//! Error types for word-tally

use std::{io, str};
use thiserror::Error;

/// Structured error types for word-tally
#[derive(Error, Debug)]
pub enum Error {
    #[error("usage: {0}")]
    Usage(String),

    #[error("memory-mapped I/O requires a file, not stdin")]
    MmapStdin,

    #[error("byte I/O mode requires `Input::from_bytes()`")]
    BytesWithPath,

    #[error("parallel bytes I/O mode requires bytes input")]
    BytesInputRequired,

    #[error("invalid UTF-8 at byte {byte}: {source}")]
    Utf8 {
        byte: usize,
        #[source]
        source: str::Utf8Error,
    },

    #[error("invalid {kind} pattern: {message}")]
    Pattern { kind: String, message: String },

    #[error("JSON serialization failed")]
    JsonSerialization(#[from] serde_json::Error),

    #[error("CSV serialization failed")]
    CsvSerialization(#[from] csv::Error),

    #[error("failed to unescape {context}: {value}")]
    Unescape { context: String, value: String },

    #[error("chunk count {chunks} exceeds platform limit of {}", usize::MAX)]
    ChunkCountExceeded { chunks: u64 },

    #[error(
        "batch size {size} bytes exceeds platform limit of {} bytes",
        usize::MAX
    )]
    BatchSizeExceeded { size: u64 },

    #[error("I/O at {path}: {message}")]
    Io {
        path: String,
        message: String,
        #[source]
        source: io::Error,
    },

    #[error("invalid configuration: {0}")]
    Config(String),

    #[error("non-ASCII byte {byte:#x} at position {position} in ASCII-only mode")]
    NonAsciiInAsciiMode { byte: u8, position: usize },
}
