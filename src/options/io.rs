//! Configuration for I/O strategies.

use clap::ValueEnum;
use core::fmt::{self, Display, Formatter};
use serde::{Deserialize, Serialize};
use std::env;

/// Determines the I/O strategy for processing input.
///
/// Performance characteristics:
///
/// **Sequential processing:**
/// - **Stream**: Single-threaded streaming with minimal memory usage.
///
/// **Parallel processing:**
/// - **`ParallelStream`**: Parallel streaming with balanced memory/performance (default).
/// - **`ParallelInMemory`**: Loads entire content into memory for parallel processing.
/// - **`ParallelMmap`**: Best performance via OS virtual memory (seekable files only).
///
/// # Examples
///
/// ```
/// use word_tally::Io;
///
/// assert_eq!(Io::default(), Io::ParallelStream);
/// assert_eq!(Io::ParallelMmap.to_string(), "parallel-mmap");
/// ```
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, ValueEnum,
)]
pub enum Io {
    /// Parallel streaming I/O — balanced memory/performance (default)
    ParallelStream,

    /// Sequential streaming I/O — minimal memory
    Stream,

    /// Parallel memory-mapped I/O — high performance but requires files, not stdin
    #[clap(alias = "mmap")]
    ParallelMmap,

    /// Parallel bytes - similar to memory-mapped I/O but directly from bytes not files
    #[clap(skip)]
    ParallelBytes,

    /// Parallel in-memory from I/O — fully loaded into memory, primarily for stdin
    ParallelInMemory,
}

impl Default for Io {
    fn default() -> Self {
        Self::ParallelStream
    }
}

impl Io {
    /// Environment variable name for I/O configuration.
    pub const ENV_IO: &'static str = "WORD_TALLY_IO";

    /// Parse I/O strategy from a string value.
    #[must_use]
    pub fn from_str_value(value: Option<&str>) -> Self {
        match value {
            Some(s) if s.eq_ignore_ascii_case("stream") => Self::Stream,
            Some(s) if s.eq_ignore_ascii_case("parallel-stream") => Self::ParallelStream,
            Some(s) if s.eq_ignore_ascii_case("parallel-in-memory") => Self::ParallelInMemory,
            Some(s)
                if s.eq_ignore_ascii_case("parallel-mmap") || s.eq_ignore_ascii_case("mmap") =>
            {
                Self::ParallelMmap
            }
            _ => Self::default(),
        }
    }

    /// Parse I/O strategy from `WORD_TALLY_IO` environment variable.
    #[must_use]
    pub fn from_env() -> Self {
        Self::from_str_value(env::var(Self::ENV_IO).ok().as_deref())
    }
}

impl Display for Io {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Stream => write!(f, "stream"),
            Self::ParallelStream => write!(f, "parallel-stream"),
            Self::ParallelInMemory => write!(f, "parallel-in-memory"),
            Self::ParallelMmap => write!(f, "parallel-mmap"),
            Self::ParallelBytes => write!(f, "parallel-bytes"),
        }
    }
}
