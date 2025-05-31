//! Thread count configuration for parallel processing.

use crate::WordTallyError;
use core::fmt::{self, Display, Formatter};
use rayon::ThreadPoolBuilder;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};

/// Thread count configuration for parallel processing.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Threads {
    /// Use all available cores.
    All,

    /// Use a specific number of threads.
    Count(u16),
}

impl Threads {
    /// Get the actual number of threads that will be used.
    #[must_use]
    pub fn count(self) -> usize {
        match self {
            Self::All => rayon::current_num_threads(),
            Self::Count(n) => n as usize,
        }
    }

    /// Initialize the Rayon thread pool.
    ///
    /// # Errors
    ///
    /// Returns an error if the thread pool cannot be initialized or if the
    /// number of threads specified is invalid.
    pub fn init_pool(self) -> anyhow::Result<()> {
        static INIT_ATTEMPTED: AtomicBool = AtomicBool::new(false);

        // Only attempt initialization once using a thread-safe check
        if INIT_ATTEMPTED.swap(true, Ordering::SeqCst) {
            return Ok(());
        }

        // Configure thread pool based on the threads setting
        match self {
            Self::Count(count) => {
                ThreadPoolBuilder::new()
                    .num_threads(count as usize)
                    .build_global()
                    .map_err(|_| {
                        WordTallyError::Config(format!(
                            "failed to configure thread pool with {count} threads"
                        ))
                    })?;
            }
            Self::All => {
                // Default Rayon behavior, no need to configure
            }
        }

        Ok(())
    }
}

impl Default for Threads {
    fn default() -> Self {
        Self::All
    }
}

impl Display for Threads {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.count())
    }
}

impl From<u16> for Threads {
    fn from(count: u16) -> Self {
        Self::Count(count)
    }
}
