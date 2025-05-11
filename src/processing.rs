//! Configuration for word tallying processing strategies.

use core::fmt::{self, Display, Formatter};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};

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

/// Thread count configuration for parallel processing
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Threads {
    /// Use all available cores
    All,

    /// Use a specific number of threads
    Count(u16),
}

impl Default for Threads {
    fn default() -> Self {
        Self::All
    }
}

impl Display for Threads {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::All => write!(f, "all"),
            Self::Count(count) => write!(f, "{}", count),
        }
    }
}

impl From<u16> for Threads {
    fn from(count: u16) -> Self {
        Self::Count(count)
    }
}

/// Represents a size hint for input data to optimize capacity allocation
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum SizeHint {
    /// No size hint available (use default capacity)
    None,

    /// Size hint in bytes
    Bytes(usize),
}

impl Default for SizeHint {
    fn default() -> Self {
        Self::None
    }
}

impl Display for SizeHint {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => write!(f, "none"),
            Self::Bytes(size) => write!(f, "{} bytes", size),
        }
    }
}

impl From<Option<usize>> for SizeHint {
    fn from(opt: Option<usize>) -> Self {
        opt.map_or(Self::None, Self::Bytes)
    }
}

impl From<SizeHint> for Option<usize> {
    fn from(hint: SizeHint) -> Self {
        match hint {
            SizeHint::Bytes(size) => Some(size),
            SizeHint::None => None,
        }
    }
}

/// Environment variable names for configuration
pub const ENV_PROCESSING: &str = "WORD_TALLY_PROCESSING";
pub const ENV_THREADS: &str = "WORD_TALLY_THREADS";

/// Parse `WORD_TALLY_PROCESSING` environment variable
pub fn parse_processing_from_env() -> Processing {
    match std::env::var(ENV_PROCESSING).ok().as_deref() {
        Some(s) if s.eq_ignore_ascii_case("sequential") => Processing::Sequential,
        Some(s) if s.eq_ignore_ascii_case("parallel") => Processing::Parallel,
        _ => Processing::default(),
    }
}

/// Parse thread count from `WORD_TALLY_THREADS` environment variable
pub fn parse_threads_from_env() -> Threads {
    std::env::var(ENV_THREADS)
        .ok()
        .and_then(|val| {
            if val.eq_ignore_ascii_case("all") {
                Some(Threads::All)
            } else {
                val.parse::<u16>().ok().map(Threads::Count)
            }
        })
        .unwrap_or_default()
}

/// Initialize the Rayon thread pool with configuration from `Threads`.
///
/// - Only attempts initialization once (thread-safe with AtomicBool)
/// - Configures thread count based on `Threads` enum value
/// - Handles initialization failures with defaults
pub fn init_thread_pool(threads: Threads, verbose: bool) {
    use rayon::ThreadPoolBuilder;
    use std::cmp::max;

    static INIT_ATTEMPTED: AtomicBool = AtomicBool::new(false);

    // Only attempt initialization once using a thread-safe check
    if INIT_ATTEMPTED.swap(true, Ordering::SeqCst) {
        return;
    }

    // Configure thread pool based on the threads setting
    match threads {
        Threads::Count(0) => {
            // For zero threads, configure with 1 thread
            if let Err(e) = ThreadPoolBuilder::new().num_threads(1).build_global() {
                if verbose {
                    eprintln!(
                        "Warning: Failed to configure single-threaded pool ({}). Using default configuration.",
                        e
                    );
                }
            }
        }
        Threads::Count(count) => {
            // Convert to usize and ensure at least one thread
            let thread_count = max(1, count as usize);

            // Only configure if different from default (all available)
            let available_parallelism = std::thread::available_parallelism()
                .map(|p| p.get())
                .unwrap_or(1);

            if thread_count < available_parallelism {
                if let Err(e) = ThreadPoolBuilder::new()
                    .num_threads(thread_count)
                    .build_global()
                {
                    // Log error but continue with default thread pool
                    if verbose {
                        eprintln!(
                            "Warning: Failed to configure thread pool with {} threads ({}). Using default configuration.",
                            thread_count, e
                        );
                    }
                } else if verbose {
                    eprintln!("Thread pool configured with {} threads", thread_count);
                }
            }
        }
        Threads::All => {
            // Default Rayon behavior, no need to configure
        }
    }
}
