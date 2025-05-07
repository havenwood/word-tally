//! Configuration for word tallying performance.

use clap::ValueEnum;
use core::fmt::{self, Display, Formatter};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};

/// Determines the I/O strategy for input
///
/// Performance characteristics:
/// - **Streamed**: Processes input line-by-line without loading entire file into memory.
///   Well-suited for memory-constrained environments and pipes/non-seekable sources.
///
/// - **Buffered**: Loads entire content into memory before processing.
///   Useful alternative when memory-mapping isn't optimal but memory is available.
///
/// - **MemoryMapped**: Uses the OS virtual memory system for efficient file access.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, ValueEnum,
)]
pub enum Io {
    /// Process input line-by-line without loading entire file into memory
    Streamed,

    /// Read entire file into memory before processing
    Buffered,

    /// Use memory-mapped I/O for efficient file access
    #[clap(name = "mmap")]
    MemoryMapped,
}

/// Determines the processing strategy
///
/// Performance characteristics:
/// - **Sequential**: Single-threaded processing that works well for most inputs.
///   Similar performance across I/O methods.
///
/// - **Parallel**: Multi-threaded processing using a work-stealing thread pool.
///   Generally beneficial for larger inputs, especially when combined with
///   streamed or memory-mapped I/O.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Processing {
    /// Process input sequentially (single-threaded).
    Sequential,

    /// Process input in parallel (multi-threaded).
    Parallel,
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
        }
    }
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

/// # Anecdotal I/O and processing selection strategy
///
/// | Use Case | Suggested Strategy | Configuration |
/// |----------|-------------------|---------------|
/// | Default usage | Sequential streaming | `Io::Streamed`, `Processing::Sequential` |
/// | Memory-constrained | Streaming | `Io::Streamed`, `Processing::Sequential` |
/// | Large files with available RAM | Memory-mapped parallel | `Io::MemoryMapped`, `Processing::Parallel` |
/// | Pipes/Non-seekable sources | Streaming | `Io::Streamed`, `Processing::Sequential` |
/// | Memory-mapping restrictions | Buffered | `Io::Buffered`, `Processing::Sequential` |
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

    /// I/O strategy (streamed, buffered, or memory-mapped)
    io: Io,

    /// Processing strategy (sequential or parallel)
    processing: Processing,

    /// Size hint for input data to optimize capacity allocation
    size_hint: SizeHint,

    /// Thread configuration for parallel processing
    threads: Threads,

    /// Option to print verbose messages
    verbose: bool,

    /// Threshold for capacity reservation when merging maps
    reserve_threshold: usize,
}

/// Default configuration values
const DEFAULT_CAPACITY: usize = 1024;
const DEFAULT_UNIQUENESS_RATIO: u8 = 10;
const DEFAULT_WORD_DENSITY: u8 = 15;
const DEFAULT_CHUNK_SIZE: u32 = 65_536; // 64KB
const DEFAULT_RESERVE_THRESHOLD: usize = 1000;

/// Environment variable names for configuration
const ENV_DEFAULT_CAPACITY: &str = "WORD_TALLY_DEFAULT_CAPACITY";
const ENV_UNIQUENESS_RATIO: &str = "WORD_TALLY_UNIQUENESS_RATIO";
const ENV_WORD_DENSITY: &str = "WORD_TALLY_WORD_DENSITY";
const ENV_CHUNK_SIZE: &str = "WORD_TALLY_CHUNK_SIZE";
const ENV_THREADS: &str = "WORD_TALLY_THREADS";
const ENV_IO: &str = "WORD_TALLY_IO";
const ENV_PROCESSING: &str = "WORD_TALLY_PROCESSING";
const ENV_VERBOSE: &str = "WORD_TALLY_VERBOSE";
const ENV_RESERVE_THRESHOLD: &str = "WORD_TALLY_RESERVE_THRESHOLD";

/// Configuration struct for Performance parameters
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Default capacity for IndexMap when no size hint is available
    pub default_capacity: usize,

    /// Ratio used to estimate number of unique words based on input size
    pub uniqueness_ratio: u8,

    /// Estimated number of unique words per character in input
    pub unique_word_density: u8,

    /// Size of chunks for parallel processing (in bytes)
    pub chunk_size: u32,

    /// I/O strategy (streamed, buffered, or memory-mapped)
    pub io: Io,

    /// Processing strategy (sequential or parallel)
    pub processing: Processing,

    /// Size hint for input data to optimize capacity allocation
    pub size_hint: SizeHint,

    /// Thread configuration for parallel processing
    pub threads: Threads,

    /// Option to print verbose messages
    pub verbose: bool,

    /// Threshold for capacity reservation when merging maps
    pub reserve_threshold: usize,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            default_capacity: DEFAULT_CAPACITY,
            uniqueness_ratio: DEFAULT_UNIQUENESS_RATIO,
            unique_word_density: DEFAULT_WORD_DENSITY,
            chunk_size: DEFAULT_CHUNK_SIZE,
            io: Io::default(),
            processing: Processing::default(),
            size_hint: SizeHint::default(),
            threads: Threads::default(),
            verbose: false,
            reserve_threshold: DEFAULT_RESERVE_THRESHOLD,
        }
    }
}

impl Default for Performance {
    fn default() -> Self {
        Self::from(PerformanceConfig::default())
    }
}

impl From<PerformanceConfig> for Performance {
    fn from(config: PerformanceConfig) -> Self {
        Self {
            default_capacity: config.default_capacity,
            uniqueness_ratio: config.uniqueness_ratio,
            unique_word_density: config.unique_word_density,
            chunk_size: config.chunk_size,
            io: config.io,
            processing: config.processing,
            threads: config.threads,
            size_hint: config.size_hint,
            verbose: config.verbose,
            reserve_threshold: config.reserve_threshold,
        }
    }
}

impl Performance {
    /// Create a new performance configuration with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a performance configuration from a config struct
    pub fn from_config(config: PerformanceConfig) -> Self {
        Self::from(config)
    }

    /// Create performance configuration from environment variables if present
    ///
    /// This method checks for all supported environment variables:
    /// - `WORD_TALLY_DEFAULT_CAPACITY`: Default capacity for IndexMap (default: 1024)
    /// - `WORD_TALLY_UNIQUENESS_RATIO`: Ratio for capacity estimation (default: 10)
    /// - `WORD_TALLY_WORD_DENSITY`: Per-chunk map capacity (default: 15)
    /// - `WORD_TALLY_CHUNK_SIZE`: Chunk size for parallel processing (default: 65536, 64KB)
    /// - `WORD_TALLY_RESERVE_THRESHOLD`: Threshold for map capacity reservation (default: 1000)
    /// - `WORD_TALLY_THREADS`: Thread count (default: all available cores)
    /// - `WORD_TALLY_IO`: I/O strategy (default: streamed, options: streamed, buffered, memory-mapped)
    /// - `WORD_TALLY_PROCESSING`: Processing strategy (default: sequential, options: sequential, parallel)
    /// - `WORD_TALLY_VERBOSE`: Verbose mode (default: false, options: true/1/yes/on)
    ///
    /// Results are cached for efficiency.
    pub fn from_env() -> Self {
        use std::sync::OnceLock;

        // Parse environment variables only once and cache the result
        static CONFIG: OnceLock<Performance> = OnceLock::new();

        *CONFIG.get_or_init(|| {
            /// Parse numeric environment variable with fallback to default value
            ///
            /// Generic parser for any environment variable that can be converted from a string
            fn parse_env_var<T: std::str::FromStr>(name: &str, default: T) -> T {
                std::env::var(name)
                    .ok()
                    .and_then(|value| value.parse().ok())
                    .unwrap_or(default)
            }

            /// Parse I/O strategy from WORD_TALLY_IO environment variable
            fn parse_io() -> Io {
                match std::env::var(ENV_IO).ok().as_deref() {
                    Some(s) if s.eq_ignore_ascii_case("streamed") => Io::Streamed,
                    Some(s) if s.eq_ignore_ascii_case("buffered") => Io::Buffered,
                    Some(s)
                        if s.eq_ignore_ascii_case("memory-mapped")
                            || s.eq_ignore_ascii_case("mmap") =>
                    {
                        Io::MemoryMapped
                    }
                    _ => Io::default(),
                }
            }

            /// Parse processing strategy from WORD_TALLY_PROCESSING environment variable
            fn parse_processing() -> Processing {
                match std::env::var(ENV_PROCESSING).ok().as_deref() {
                    Some(s) if s.eq_ignore_ascii_case("sequential") => Processing::Sequential,
                    Some(s) if s.eq_ignore_ascii_case("parallel") => Processing::Parallel,
                    _ => Processing::default(),
                }
            }

            /// Parse thread count from WORD_TALLY_THREADS environment variable
            ///
            /// Returns Threads::All for "all" or Threads::Count(n) for numeric values
            fn parse_threads() -> Threads {
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

            /// Parse verbose flag from WORD_TALLY_VERBOSE environment variable
            ///
            /// Returns true for "1", "true", "yes", or "on"
            fn parse_verbose() -> bool {
                matches!(
                    std::env::var(ENV_VERBOSE).ok().as_deref(),
                    Some("1" | "true" | "yes" | "on")
                )
            }

            let config = PerformanceConfig {
                default_capacity: parse_env_var(ENV_DEFAULT_CAPACITY, DEFAULT_CAPACITY),
                uniqueness_ratio: parse_env_var(ENV_UNIQUENESS_RATIO, DEFAULT_UNIQUENESS_RATIO),
                unique_word_density: parse_env_var(ENV_WORD_DENSITY, DEFAULT_WORD_DENSITY),
                chunk_size: parse_env_var(ENV_CHUNK_SIZE, DEFAULT_CHUNK_SIZE),
                reserve_threshold: parse_env_var(ENV_RESERVE_THRESHOLD, DEFAULT_RESERVE_THRESHOLD),
                io: parse_io(),
                processing: parse_processing(),
                threads: parse_threads(),
                size_hint: SizeHint::default(),
                verbose: parse_verbose(),
            };

            Self::from(config)
        })
    }

    /// Create a new configuration with custom capacity setting
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

    /// Set the I/O strategy for this configuration
    pub const fn with_io(mut self, io: Io) -> Self {
        self.io = io;
        self
    }

    /// Set the processing strategy for this configuration
    pub const fn with_processing(mut self, processing: Processing) -> Self {
        self.processing = processing;
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

    /// Set verbose mode
    pub const fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Set the reserve threshold for capacity allocation
    pub const fn with_reserve_threshold(mut self, threshold: usize) -> Self {
        self.reserve_threshold = threshold;
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

    /// Get the I/O strategy
    pub const fn io(&self) -> Io {
        self.io
    }

    /// Get the processing strategy
    pub const fn processing(&self) -> Processing {
        self.processing
    }

    /// Check if verbose mode is enabled
    pub const fn verbose(&self) -> bool {
        self.verbose
    }

    /// Get the reserve threshold for capacity allocation
    pub const fn reserve_threshold(&self) -> usize {
        self.reserve_threshold
    }

    /// Calculate dynamic reserve threshold based on map size and configuration
    pub fn calc_reserve_threshold(&self) -> usize {
        let base_threshold = self.reserve_threshold;

        match self.size_hint {
            SizeHint::None => base_threshold,
            SizeHint::Bytes(size) => {
                let estimated_capacity = (size / self.uniqueness_ratio as u64) as usize;
                base_threshold.max(estimated_capacity / 10)
            }
        }
    }

    /// Estimate overall map capacity based on input size
    pub const fn estimate_capacity(&self) -> usize {
        match self.size_hint {
            SizeHint::None => self.default_capacity,
            SizeHint::Bytes(size) => (size / self.uniqueness_ratio as u64) as usize,
        }
    }

    /// Estimate thread-local map capacity for parallel processing
    pub fn estimate_thread_local_capacity(&self) -> usize {
        let threads = match self.threads {
            Threads::All => rayon::current_num_threads(),
            Threads::Count(n) => n as usize,
        };

        let base_capacity = self.estimate_capacity();
        let max_threads = threads.clamp(1, 128);

        base_capacity / max_threads
    }

    /// Estimate chunk map capacity based on chunk size
    pub const fn estimate_chunk_capacity(&self, chunk_size: usize) -> usize {
        (chunk_size * self.unique_word_density as usize) / 10 + 10
    }

    /// Initialize the Rayon thread pool with configuration from `Performance`.
    ///
    /// - Only attempts initialization once (thread-safe with AtomicBool)
    /// - Configures thread count based on Threads enum value
    /// - Handles initialization failures with defaults
    /// - Avoids reinitializing the global thread pool
    pub fn init_thread_pool(&self) {
        use rayon::ThreadPoolBuilder;
        use std::cmp::max;

        static INIT_ATTEMPTED: AtomicBool = AtomicBool::new(false);

        // Only attempt initialization once using a thread-safe check
        if INIT_ATTEMPTED.swap(true, Ordering::SeqCst) {
            return;
        }

        // Configure thread pool based on the threads setting
        match self.threads {
            Threads::Count(0) => {
                // For zero threads, configure with 1 thread
                if let Err(e) = ThreadPoolBuilder::new().num_threads(1).build_global() {
                    if self.verbose {
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
                        if self.verbose {
                            eprintln!(
                                "Warning: Failed to configure thread pool with {} threads ({}). Using default configuration.",
                                thread_count, e
                            );
                        }
                    } else if self.verbose {
                        eprintln!("Thread pool configured with {} threads", thread_count);
                    }
                }
            }
            Threads::All => {
                // Default Rayon behavior, no need to configure
            }
        }
    }
}
