//! Configuration for I/O strategies.

use core::fmt::{self, Display, Formatter};
use std::{env, str::FromStr};

use clap::ValueEnum;
use serde::{Deserialize, Serialize};

use crate::FileType;

/// Determines the I/O strategy for processing input.
///
/// Performance characteristics:
///
/// **Automatic selection (default):**
/// - **`Auto`**: Automatically selects a reasonable I/O strategy based on input type.
///
/// **Sequential processing:**
/// - **`Stream`**: Single-threaded streaming with minimal memory usage.
///
/// **Parallel processing:**
/// - **`ParallelStream`**: Parallel streaming with balanced memory/performance.
/// - **`ParallelInMemory`**: Loads entire content into memory for parallel processing.
/// - **`ParallelMmap`**: Best performance via OS virtual memory (seekable files only).
///
/// # Examples
///
/// ```
/// use word_tally::{Buffered, Io, Mapped};
///
/// # fn example() -> anyhow::Result<()> {
/// // Default: reader for files and stdin
/// let reader = Buffered::try_from("file.txt")?;
///
/// // Memory-mapped files for maximum speed
/// let view = Mapped::try_from("large_file.txt")?;
///
/// // Stdin processing
/// let stdin = Buffered::stdin();
/// // or
/// let stdin = Buffered::try_from("-")?;
///
/// // Bytes input
/// let bytes_view = Mapped::from(b"some text data");
/// # Ok(())
/// # }
/// ```
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, ValueEnum,
)]
#[serde(rename_all = "camelCase")]
pub enum Io {
    /// Automatically select a reasonable I/O strategy based on input type (default)
    Auto,

    /// Parallel bytes - similar to memory-mapped I/O but directly from bytes not files
    #[clap(skip)]
    ParallelBytes,

    /// Parallel memory-mapped I/O — high performance but requires files, not stdin
    #[clap(alias = "mmap")]
    ParallelMmap,

    /// Parallel in-memory from I/O — fully loaded into memory, primarily for stdin
    ParallelInMemory,

    /// Parallel streaming I/O — balanced memory/performance
    ParallelStream,

    /// Sequential streaming I/O — minimal memory
    Stream,
}

impl Default for Io {
    fn default() -> Self {
        Self::Auto
    }
}

impl Io {
    /// Environment variable name for I/O configuration.
    pub const ENV_IO: &'static str = "WORD_TALLY_IO";

    /// Parse I/O strategy from a string value.
    #[must_use]
    pub fn from_str_value(value: Option<&str>) -> Self {
        value.and_then(|s| s.parse().ok()).unwrap_or_default()
    }

    /// Parse I/O strategy from `WORD_TALLY_IO` environment variable.
    #[must_use]
    pub fn from_env() -> Self {
        Self::from_str_value(env::var(Self::ENV_IO).ok().as_deref())
    }

    /// Select a reasonable I/O strategy based on file type.
    ///
    /// - Regular files: Use `ParallelMmap` for best performance
    /// - Pipes, sockets: Use `ParallelStream` for streaming data
    /// - Directories: Return `ParallelStream` (will fail later with proper error)
    /// - Character/block devices: Use `Stream` for simple sequential processing
    /// - Unknown (stdin, bytes): Default to `ParallelStream`
    #[must_use]
    pub const fn select_for_file_type(file_type: Option<FileType>) -> Self {
        match file_type {
            Some(FileType::File) => Self::ParallelMmap,
            Some(FileType::CharacterDevice | FileType::BlockDevice) => Self::Stream,
            Some(
                FileType::NamedPipe | FileType::Socket | FileType::Directory | FileType::Special,
            )
            | None => Self::ParallelStream,
        }
    }

    /// Resolve Auto mode to a concrete I/O strategy.
    #[must_use]
    pub const fn resolve(self, file_type: Option<FileType>) -> Self {
        match self {
            Self::Auto => Self::select_for_file_type(file_type),
            other => other,
        }
    }
}

impl Display for Io {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Auto => write!(f, "auto"),
            Self::Stream => write!(f, "stream"),
            Self::ParallelStream => write!(f, "parallel-stream"),
            Self::ParallelInMemory => write!(f, "parallel-in-memory"),
            Self::ParallelMmap => write!(f, "parallel-mmap"),
            Self::ParallelBytes => write!(f, "parallel-bytes"),
        }
    }
}

impl FromStr for Io {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            _ if s.eq_ignore_ascii_case("auto") => Self::Auto,
            _ if s.eq_ignore_ascii_case("stream") => Self::Stream,
            _ if s.eq_ignore_ascii_case("parallel-stream") => Self::ParallelStream,
            _ if s.eq_ignore_ascii_case("parallel-in-memory") => Self::ParallelInMemory,
            _ if s.eq_ignore_ascii_case("parallel-mmap") || s.eq_ignore_ascii_case("mmap") => {
                Self::ParallelMmap
            }
            _ if s.eq_ignore_ascii_case("parallel-bytes") => Self::ParallelBytes,
            _ => return Err(()),
        })
    }
}
