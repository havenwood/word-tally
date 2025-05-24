//! Configuration for word tallying processing strategies.

use core::fmt::{self, Display, Formatter};
use serde::{Deserialize, Serialize};

use crate::Performance;

/// Determines the processing strategy.
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
/// assert_eq!(Processing::default(), Processing::Parallel);
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
        Self::Parallel
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

impl Processing {
    /// Initialize resources needed for this processing mode.
    ///
    /// For parallel processing, this initializes the global thread pool.
    /// For sequential processing, this is a no-op.
    ///
    /// # Errors
    ///
    /// Returns an error if parallel mode is selected but the thread pool
    /// cannot be initialized.
    pub fn initialize(&self, performance: &Performance) -> anyhow::Result<()> {
        match self {
            Self::Parallel => performance.threads().init_pool(),
            Self::Sequential => Ok(()),
        }
    }
}

impl From<bool> for Processing {
    fn from(parallel: bool) -> Self {
        if parallel {
            Self::Parallel
        } else {
            Self::Sequential
        }
    }
}
