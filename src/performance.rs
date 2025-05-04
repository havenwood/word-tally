//! Configuration for word tallying performance.

use core::fmt::{self, Display, Formatter};
use serde::{Deserialize, Serialize};

/// Determines whether to use parallel or sequential processing
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Concurrency {
    /// Process input sequentially
    Sequential,

    /// Process input in parallel
    Parallel,
}

impl Default for Concurrency {
    fn default() -> Self {
        Self::Sequential
    }
}

impl Display for Concurrency {
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
    Bytes(u64),
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

impl From<Option<u64>> for SizeHint {
    fn from(opt: Option<u64>) -> Self {
        opt.map_or(Self::None, Self::Bytes)
    }
}

impl From<SizeHint> for Option<u64> {
    fn from(hint: SizeHint) -> Self {
        match hint {
            SizeHint::Bytes(size) => Some(size),
            SizeHint::None => None,
        }
    }
}

/// Configuration for word tallying performance
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Performance {
    /// Default capacity for IndexMap when no size hint is available
    default_capacity: usize,

    /// Ratio used to estimate number of unique words based on input size
    uniqueness_ratio: u8,

    /// Estimated number of unique words per character in input
    unique_word_density: u8,

    /// Size of chunks for parallel processing (in bytes)
    chunk_size: u32,

    /// Whether to process sequentially or in parallel
    concurrency: Concurrency,

    /// Size hint for input data to optimize capacity allocation
    size_hint: SizeHint,

    /// Thread configuration for parallel processing
    threads: Threads,
}

/// Default configuration values
const DEFAULT_CAPACITY: usize = 1024;
const DEFAULT_UNIQUENESS_RATIO: u8 = 10;
const DEFAULT_WORD_DENSITY: u8 = 15;
const DEFAULT_CHUNK_SIZE: u32 = 16_384; // 16KB default

/// Environment variable names for configuration
const ENV_DEFAULT_CAPACITY: &str = "WORD_TALLY_DEFAULT_CAPACITY";
const ENV_UNIQUENESS_RATIO: &str = "WORD_TALLY_UNIQUENESS_RATIO";
const ENV_WORD_DENSITY: &str = "WORD_TALLY_WORD_DENSITY";
const ENV_CHUNK_SIZE: &str = "WORD_TALLY_CHUNK_SIZE";
const ENV_THREADS: &str = "WORD_TALLY_THREADS";

impl Default for Performance {
    fn default() -> Self {
        Self {
            default_capacity: DEFAULT_CAPACITY,
            uniqueness_ratio: DEFAULT_UNIQUENESS_RATIO,
            unique_word_density: DEFAULT_WORD_DENSITY,
            chunk_size: DEFAULT_CHUNK_SIZE,
            concurrency: Concurrency::default(),
            size_hint: SizeHint::default(),
            threads: Threads::default(),
        }
    }
}

impl Performance {
    /// Create a new performance configuration for a word tally
    pub const fn new(
        default_capacity: usize,
        uniqueness_ratio: u8,
        unique_word_density: u8,
        chunk_size: u32,
        concurrency: Concurrency,
        threads: Threads,
        size_hint: SizeHint,
    ) -> Self {
        Self {
            default_capacity,
            uniqueness_ratio,
            unique_word_density,
            chunk_size,
            concurrency,
            threads,
            size_hint,
        }
    }

    /// Create performance configuration from common environment variables if present
    ///
    /// This loads the non-parallel specific environment variables.
    /// Parallel-specific environment variables are loaded in `with_concurrency`.
    pub fn from_env() -> Self {
        use std::sync::OnceLock;

        // Parse environment variables only once and cache the result
        static CONFIG: OnceLock<Performance> = OnceLock::new();

        *CONFIG.get_or_init(|| {
            fn parse_env_var<T: std::str::FromStr>(name: &str, default: T) -> T {
                std::env::var(name)
                    .ok()
                    .and_then(|value| value.parse().ok())
                    .unwrap_or(default)
            }

            Self {
                // Load common environment variables
                default_capacity: parse_env_var(ENV_DEFAULT_CAPACITY, DEFAULT_CAPACITY),
                uniqueness_ratio: parse_env_var(ENV_UNIQUENESS_RATIO, DEFAULT_UNIQUENESS_RATIO),

                // Default values for parallel-specific settings
                // These will be loaded in with_concurrency if needed
                unique_word_density: DEFAULT_WORD_DENSITY,
                chunk_size: DEFAULT_CHUNK_SIZE,

                concurrency: Concurrency::default(),
                size_hint: SizeHint::default(),
                threads: Threads::default(),
            }
        })
    }

    /// Create a new configuration with custom settings
    pub const fn with_capacity(mut self, capacity: usize) -> Self {
        self.default_capacity = capacity;
        self
    }

    /// Set the uniqueness ratio for this configuration
    pub const fn with_uniqueness_ratio(mut self, ratio: u8) -> Self {
        self.uniqueness_ratio = ratio;
        self
    }

    /// Set the word density for this configuration
    pub const fn with_word_density(mut self, density: u8) -> Self {
        self.unique_word_density = density;
        self
    }

    /// Set the chunk size for this configuration
    pub const fn with_chunk_size(mut self, size: u32) -> Self {
        self.chunk_size = size;
        self
    }

    /// Set the concurrency mode for this configuration and load parallelism
    /// environment variables like WORD_TALLY_CHUNK_SIZE, WORD_TALLY_WORD_DENSITY,
    /// and WORD_TALLY_THREADS.
    pub fn with_concurrency(mut self, concurrency: Concurrency) -> Self {
        self.concurrency = concurrency;

        if matches!(concurrency, Concurrency::Parallel) {
            // Update chunk size if env var exists and can be parsed
            if let Some(size) = std::env::var(ENV_CHUNK_SIZE)
                .ok()
                .and_then(|val| val.parse().ok())
            {
                self.chunk_size = size;
            }

            // Update word density if env var exists and can be parsed
            if let Some(density) = std::env::var(ENV_WORD_DENSITY)
                .ok()
                .and_then(|val| val.parse().ok())
            {
                self.unique_word_density = density;
            }

            // Update thread count if env var exists and can be parsed
            if let Some(threads) = std::env::var(ENV_THREADS)
                .ok()
                .and_then(|val| val.parse::<u16>().ok())
            {
                self.threads = Threads::Count(threads);
            }
        }

        self
    }

    /// Set the size hint for this configuration
    pub const fn with_size_hint(mut self, size_hint: SizeHint) -> Self {
        self.size_hint = size_hint;
        self
    }

    /// Set the thread configuration
    pub const fn with_threads(mut self, threads: Threads) -> Self {
        self.threads = threads;
        self
    }

    /// Get the default capacity for hash maps
    pub const fn default_capacity(&self) -> usize {
        self.default_capacity
    }

    /// Get the uniqueness ratio used for capacity estimation
    pub const fn uniqueness_ratio(&self) -> u8 {
        self.uniqueness_ratio
    }

    /// Get the unique word density used for chunk capacity estimation
    pub const fn unique_word_density(&self) -> u8 {
        self.unique_word_density
    }

    /// Get the chunk size for parallel processing
    pub const fn chunk_size(&self) -> u32 {
        self.chunk_size
    }

    /// Get the size hint
    pub const fn size_hint(&self) -> SizeHint {
        self.size_hint
    }

    /// Get the thread configuration
    pub const fn threads(&self) -> Threads {
        self.threads
    }

    /// Get the concurrency mode
    pub const fn concurrency(&self) -> Concurrency {
        self.concurrency
    }

    /// Estimate map capacity based on input size
    pub const fn estimate_capacity(&self) -> usize {
        match self.size_hint {
            SizeHint::None => self.default_capacity,
            SizeHint::Bytes(size) => (size / self.uniqueness_ratio as u64) as usize,
        }
    }

    /// Estimate chunk map capacity based on chunk size
    pub const fn estimate_chunk_capacity(&self, chunk_size: usize) -> usize {
        chunk_size * self.unique_word_density as usize
    }

    /// Initializes the rayon thread pool with configuration from Performance
    pub fn init_thread_pool(&self) {
        use rayon::ThreadPoolBuilder;
        use std::sync::Once;
        static INIT_THREAD_POOL: Once = Once::new();

        match self.threads {
            Threads::Count(count) => {
                INIT_THREAD_POOL.call_once(|| {
                    let num_threads = count as usize;
                    if let Err(e) = ThreadPoolBuilder::new()
                        .num_threads(num_threads)
                        .build_global()
                    {
                        eprintln!("Warning: Failed to set thread pool size: {}", e);
                    }
                });
            }
            Threads::All => {
                // Default rayon behavior is to use all available cores
                INIT_THREAD_POOL.call_once(|| {
                    // No custom configuration needed - Rayon defaults to all cores
                });
            }
        }
    }
}
