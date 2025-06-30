//! Error types for word-tally.

use std::io;

use thiserror::Error;

use crate::input::FileType;

/// Format an I/O error message with optional file type.
fn format_io_error(message: &str, file_type: Option<FileType>) -> String {
    file_type.map_or_else(|| message.to_string(), |ft| format!("{message} ({ft})"))
}

/// Structured error types for word-tally
#[derive(Error, Debug)]
pub enum Error {
    /// Invalid command-line usage.
    #[error("usage: {0}")]
    Usage(String),

    /// Memory-mapped I/O attempted on stdin.
    #[error("cannot memory map stdin")]
    StdinNotMappable,

    /// Memory-mapped I/O attempted on non-mappable file type.
    #[error("cannot memory map {file_type}: {path}")]
    NotMappable {
        /// Type of file that cannot be mapped.
        file_type: FileType,
        /// Path to the file.
        path: String,
    },

    /// Byte I/O mode used with file path.
    #[error("byte I/O mode requires bytes input")]
    PathInvalid,

    /// Parallel bytes mode requires bytes input.
    #[error("parallel bytes I/O mode requires bytes input")]
    BytesRequired,

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
    Json(#[from] serde_json::Error),

    /// CSV serialization error.
    #[error("CSV serialization failed")]
    Csv(#[from] csv::Error),

    /// Chunk count exceeds platform limits.
    #[error("chunk count {chunks} exceeds platform limit of {}", usize::MAX)]
    ChunkOverflow {
        /// Number of chunks requested.
        chunks: u64,
    },

    /// Batch size exceeds platform limits.
    #[error(
        "batch size {size} bytes exceeds platform limit of {} bytes",
        usize::MAX
    )]
    BatchOverflow {
        /// Batch size in bytes.
        size: u64,
    },

    /// I/O error with context.
    #[error("{}: {}", format_io_error(message, *file_type), path)]
    Io {
        /// File path where error occurred.
        path: String,
        /// Error description.
        message: String,
        /// Type of file, if known.
        file_type: Option<FileType>,
        /// Underlying I/O error.
        #[source]
        source: io::Error,
    },

    /// Runtime configuration error.
    #[error("runtime configuration error: {0}")]
    Config(String),

    /// Internal error: mutex was poisoned.
    #[error("internal error: reader mutex was poisoned")]
    MutexPoisoned,
}
