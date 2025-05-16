//! Configuration for word tallying processing strategies.

use core::fmt::{self, Display, Formatter};
use serde::{Deserialize, Serialize};

/// Determines the processing strategy
///
/// Performance characteristics:
/// - **Sequential**: Single-threaded processing with less overhead.
///   May be faster and use less memory for small input sizes.
///
/// - **Parallel**: Multi-threaded processing using a work-stealing thread pool.
///   Uses multiple core to become faster than sequential as input size grows.
///
/// # Examples
///
/// ```
/// use word_tally::Processing;
///
/// assert_eq!(Processing::default(), Processing::Sequential);
/// assert_eq!(Processing::Parallel.to_string(), "parallel");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Processing {
    /// Process input sequentially (single-threaded).
    Sequential,

    /// Process input in parallel (multi-threaded).
    Parallel,
}

impl Default for Processing {
    fn default() -> Self {
        Self::Sequential
    }
}

impl Display for Processing {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sequential => write!(f, "sequential"),
            Self::Parallel => write!(f, "parallel"),
        }
    }
}

/// Environment variable names for configuration
pub const ENV_PROCESSING: &str = "WORD_TALLY_PROCESSING";

/// Parse `WORD_TALLY_PROCESSING` environment variable
pub fn parse_processing_from_env() -> Processing {
    match std::env::var(ENV_PROCESSING).ok().as_deref() {
        Some(s) if s.eq_ignore_ascii_case("sequential") => Processing::Sequential,
        Some(s) if s.eq_ignore_ascii_case("parallel") => Processing::Parallel,
        _ => Processing::default(),
    }
}
