//! Configuration for I/O strategies.

use clap::ValueEnum;
use core::fmt::{self, Display, Formatter};
use serde::{Deserialize, Serialize};
use std::env;

/// Determines the I/O strategy for processing input, all either in parallel or sequentially.
///
/// Performance characteristics:
/// - **Streamed**: Processes input line-by-line without loading entire file into memory.
///   Well-suited for memory-constrained environments and pipes/non-seekable sources.
///
/// - **Buffered**: Loads entire content into memory before processing.
///   Useful alternative when memory-mapping isn't optimal but memory is available.
///
/// - **MemoryMapped**: Uses the OS virtual memory system for efficient file access.
///   Requires a seekable file. Raises an error with piped input like stdin.
///
/// # Examples
///
/// ```
/// use word_tally::Io;
///
/// assert_eq!(Io::default(), Io::Streamed);
/// assert_eq!(Io::MemoryMapped.to_string(), "memory-mapped");
/// ```
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, ValueEnum,
)]
pub enum Io {
    /// Use memory-mapped I/O for efficient file access
    #[clap(name = "mmap")]
    MemoryMapped,

    /// Process input line-by-line without loading entire file into memory
    Streamed,

    /// Read entire file into memory before processing
    Buffered,

    /// Process bytes directly from memory without file I/O
    #[clap(skip)]
    Bytes,
}

impl Default for Io {
    fn default() -> Self {
        Self::Streamed
    }
}

impl Display for Io {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Streamed => write!(f, "streamed"),
            Self::Buffered => write!(f, "buffered"),
            Self::MemoryMapped => write!(f, "memory-mapped"),
            Self::Bytes => write!(f, "bytes"),
        }
    }
}

/// Environment variable name for I/O configuration
pub const ENV_IO: &str = "WORD_TALLY_IO";

/// Parse I/O strategy from WORD_TALLY_IO environment variable
pub fn parse_io_from_env() -> Io {
    match env::var(ENV_IO).ok().as_deref() {
        Some(s) if s.eq_ignore_ascii_case("streamed") => Io::Streamed,
        Some(s) if s.eq_ignore_ascii_case("buffered") => Io::Buffered,
        Some(s) if s.eq_ignore_ascii_case("memory-mapped") || s.eq_ignore_ascii_case("mmap") => {
            Io::MemoryMapped
        }
        _ => Io::default(),
    }
}
