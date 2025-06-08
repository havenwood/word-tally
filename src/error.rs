//! Error types for word-tally.

use std::io;
use thiserror::Error;

/// Structured error types for word-tally
#[derive(Error, Debug)]
pub enum Error {
    /// Invalid command-line usage.
    #[error("usage: {0}")]
    Usage(String),

    /// Memory-mapped I/O attempted on stdin.
    #[error("memory-mapped I/O requires a file, not stdin")]
    MmapStdin,

    /// Byte I/O mode used with file path.
    #[error("byte I/O mode requires `Input::from()`")]
    BytesWithPath,

    /// Parallel bytes mode requires bytes input.
    #[error("parallel bytes I/O mode requires bytes input")]
    BytesInputRequired,

    /// UTF-8 decoding error.
    #[error("invalid UTF-8 at byte {byte}: {message}")]
    Utf8 {
        /// Byte position of invalid UTF-8.
        byte: usize,
        /// Error message.
        message: String,
    },

    /// Invalid regex pattern.
    #[error("invalid {kind} pattern: {message}")]
    Pattern {
        /// Pattern type (include/exclude).
        kind: String,
        /// Error message.
        message: String,
    },

    /// JSON serialization error.
    #[error("JSON serialization failed")]
    JsonSerialization(#[from] serde_json::Error),

    /// CSV serialization error.
    #[error("CSV serialization failed")]
    CsvSerialization(#[from] csv::Error),

    /// Chunk count exceeds platform limits.
    #[error("chunk count {chunks} exceeds platform limit of {}", usize::MAX)]
    ChunkCountExceeded {
        /// Number of chunks requested.
        chunks: u64,
    },

    /// Batch size exceeds platform limits.
    #[error(
        "batch size {size} bytes exceeds platform limit of {} bytes",
        usize::MAX
    )]
    BatchSizeExceeded {
        /// Batch size in bytes.
        size: u64,
    },

    /// I/O error with context.
    #[error("{message}: {path}")]
    Io {
        /// File path where error occurred.
        path: String,
        /// Error description.
        message: String,
        /// Underlying I/O error.
        #[source]
        source: io::Error,
    },

    /// Configuration error.
    #[error("invalid configuration: {0}")]
    Config(String),

    /// Non-ASCII byte in ASCII-only mode.
    #[error("non-ASCII byte {byte:#x} at position {position} in ASCII-only mode")]
    NonAsciiInAsciiMode {
        /// The non-ASCII byte value.
        byte: u8,
        /// Byte position in input.
        position: usize,
    },
}
