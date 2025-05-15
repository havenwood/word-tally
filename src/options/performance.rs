//! Configuration for word tallying performance.

use crate::options::processing::{SizeHint, Threads, init_thread_pool, parse_threads_from_env};
use core::fmt;
use serde::{Deserialize, Serialize};

/// # Performance tuning configuration parameters
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Performance {
    /// Default capacity for `TallyMap` (`IndexMap`) when no size hint is available
    default_capacity: usize,

    /// Ratio used to estimate number of unique words based on input size
    uniqueness_ratio: u8,

    /// Words-per-KB of text
    words_per_kb: u8,

    /// Size of chunks for parallel processing (in bytes)
    chunk_size: usize,

    /// Size hint for input data to optimize capacity allocation
    size_hint: SizeHint,

    /// Thread configuration for parallel processing
    threads: Threads,

    /// Option to print verbose messages
    verbose: bool,
}

/// Default configuration values
pub const TALLY_MAP_CAPACITY: usize = 16_384;
pub const CHARS_PER_LINE: usize = 80; // Chars per line estimation
const CHUNK_SIZE: usize = 65_536; // 64KB
const UNIQUENESS_RATIO: u8 = 10; // 10:1 ratio
const WORDS_PER_KB: u8 = 200; // words-per-KB estimation

/// Environment variable names for configuration
const ENV_TALLY_MAP_CAPACITY: &str = "WORD_TALLY_TALLY_MAP_CAPACITY";
const ENV_CHUNK_SIZE: &str = "WORD_TALLY_CHUNK_SIZE";
const ENV_UNIQUENESS_RATIO: &str = "WORD_TALLY_UNIQUENESS_RATIO";
const ENV_WORDS_PER_KB: &str = "WORD_TALLY_WORDS_PER_KB";
const ENV_VERBOSE: &str = "WORD_TALLY_VERBOSE";

/// Configuration struct for `Performance` parameters
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Default capacity for `TallyMap` (`IndexMap`) when no size hint is available
    pub default_capacity: usize,

    /// Ratio used to estimate number of unique words based on input size
    pub uniqueness_ratio: u8,

    /// Words-per-KB of text
    pub words_per_kb: u8,

    /// Size of chunks for parallel processing (in bytes)
    pub chunk_size: usize,

    /// Size hint for input data to optimize capacity allocation
    pub size_hint: SizeHint,

    /// Thread configuration for parallel processing
    pub threads: Threads,

    /// Option to print verbose messages
    pub verbose: bool,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            default_capacity: TALLY_MAP_CAPACITY,
            uniqueness_ratio: UNIQUENESS_RATIO,
            words_per_kb: WORDS_PER_KB,
            chunk_size: CHUNK_SIZE,
            size_hint: SizeHint::default(),
            threads: Threads::default(),
            verbose: false,
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
            words_per_kb: config.words_per_kb,
            chunk_size: config.chunk_size,
            threads: config.threads,
            size_hint: config.size_hint,
            verbose: config.verbose,
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
                default_capacity: parse_env_var(ENV_TALLY_MAP_CAPACITY, TALLY_MAP_CAPACITY),
                uniqueness_ratio: parse_env_var(ENV_UNIQUENESS_RATIO, UNIQUENESS_RATIO),
                words_per_kb: parse_env_var(ENV_WORDS_PER_KB, WORDS_PER_KB),
                chunk_size: parse_env_var(ENV_CHUNK_SIZE, CHUNK_SIZE),
                threads: parse_threads_from_env(),
                size_hint: SizeHint::default(),
                verbose: parse_verbose(),
            };

            Self::from(config)
        })
    }

    /// Create a new configuration with custom TallyMap capacity setting
    pub const fn with_tally_map_capacity(mut self, capacity: usize) -> Self {
        self.default_capacity = capacity;
        self
    }

    /// Set the uniqueness ratio for this configuration
    pub const fn with_uniqueness_ratio(mut self, ratio: u8) -> Self {
        self.uniqueness_ratio = ratio;
        self
    }

    /// Set the words-per-KB for this configuration
    pub const fn with_words_per_kb(mut self, words_per_kb: u8) -> Self {
        self.words_per_kb = words_per_kb;
        self
    }

    /// Set the chunk size for this configuration
    pub const fn with_chunk_size(mut self, size: usize) -> Self {
        self.chunk_size = size;
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

    /// Get the default capacity for TallyMap
    pub const fn default_tally_map_capacity(&self) -> usize {
        self.default_capacity
    }

    /// Get the uniqueness ratio used for capacity estimation
    pub const fn uniqueness_ratio(&self) -> u8 {
        self.uniqueness_ratio
    }

    /// Get the words-per-KB used for chunk capacity estimation
    pub const fn words_per_kb(&self) -> u8 {
        self.words_per_kb
    }

    /// Get the chunk size for parallel processing
    pub const fn chunk_size(&self) -> usize {
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

    /// Estimate the appropriate thread pool size for parallel processing
    pub fn estimate_thread_pool_size(&self) -> usize {
        match self.threads {
            Threads::All => rayon::current_num_threads(),
            Threads::Count(n) => n as usize,
        }
    }

    /// Check if verbose mode is enabled
    pub const fn verbose(&self) -> bool {
        self.verbose
    }

    /// Calculate `TallyMap` capacity based on input size
    ///
    /// Determines the optimal initial capacity for the `TallyMap`:
    /// - With size hint: Calculates based on input size, words-per-KB, and uniqueness ratio
    /// - Without size hint: Uses the default tally map capacity
    pub const fn tally_map_capacity(&self) -> usize {
        match self.size_hint {
            SizeHint::None => self.default_capacity,
            SizeHint::Bytes(size) => self.tally_map_capacity_for_content(size),
        }
    }

    /// Calculate per-thread `TallyMap` capacity for parallel processing
    ///
    /// Determines an appropriate capacity for thread-local maps that
    /// balances memory usage while ensuring good performance.
    pub fn per_thread_tally_map_capacity(&self) -> usize {
        let num_threads = match self.threads {
            Threads::All => rayon::current_num_threads().max(1),
            Threads::Count(n) => n.max(1) as usize,
        };

        // For smaller thread counts, allocate more per thread
        // For larger thread counts, avoid excessive memory usage
        // Ensure a minimum capacity to avoid excessive rehashing
        let base_capacity = self.default_capacity / num_threads;
        let min_capacity = 1024;

        base_capacity.max(min_capacity)
    }

    /// Calculate capacity for text chunk containers (`Vec<Vec<u8>>`)
    ///
    /// Estimates how many words might be in a chunk of text based on:
    /// - The chunk size in bytes
    /// - The words-per-KB value
    pub const fn text_chunk_capacity(&self) -> usize {
        // Convert chunk size to KB and multiply by words per KB
        (self.chunk_size / 1024) * self.words_per_kb as usize
    }

    /// Calculate capacity for a `TallyMap` based on content length
    ///
    /// Estimates the number of unique words that might be in a text of given length:
    /// - Calculates total words based on words-per-KB ratio
    /// - Divides by uniqueness ratio to get estimated unique words
    pub const fn tally_map_capacity_for_content(&self, content_length: usize) -> usize {
        // Convert to KB first to avoid potential integer overflow or underflow
        let kb_size = content_length / 1024;
        let estimated_words = kb_size * self.words_per_kb as usize;
        estimated_words / self.uniqueness_ratio as usize
    }

    /// Calculate tally map capacity for buffered input
    ///
    /// Uses words-per-KB and uniqueness ratio to estimate capacity needs:
    /// - Converts content size to KB
    /// - Estimates total words based on words-per-KB ratio
    /// - Estimates unique words using uniqueness ratio
    pub const fn tally_map_capacity_for_buffered(&self, content_size: usize) -> usize {
        // Calculate words estimate using KB size
        let kb_size = content_size / 1024;
        let total_words_estimate = kb_size * self.words_per_kb as usize;

        total_words_estimate / self.uniqueness_ratio as usize
    }

    /// Calculate capacity for a `TallyMap` based on line count
    ///
    /// Estimates the number of unique words based on:
    /// - Line count and average characters per line
    /// - Words-per-KB and uniqueness ratio
    pub const fn tally_map_capacity_for_lines(
        &self,
        line_count: usize,
        chars_per_line: usize,
    ) -> usize {
        let content_size = line_count * chars_per_line;
        self.tally_map_capacity_for_content(content_size)
    }

    /// Estimates how many lines in a text based on:
    /// - Total content length
    /// - Average characters per line
    pub const fn lines_vector_capacity(
        &self,
        content_length: usize,
        chars_per_line: usize,
    ) -> usize {
        content_length / chars_per_line
    }

    /// Estimates a good capacity for maps processing line chunks:
    /// - Based on chunk size in lines
    /// - Applies the uniqueness ratio to estimate unique words
    pub const fn chunk_tally_map_capacity(&self, chunk_size_in_lines: usize) -> usize {
        // Assume average 10 words per line and apply uniqueness ratio
        let words_in_chunk = chunk_size_in_lines * UNIQUENESS_RATIO as usize;
        let unique_words_estimate = words_in_chunk / self.uniqueness_ratio as usize;
        if unique_words_estimate < 64 {
            64 // Ensure a reasonable minimum capacity
        } else {
            unique_words_estimate
        }
    }

    /// Estimates a suitable initial capacity for the reduction phase map:
    /// - Based on chunk size and estimated unique words per chunk
    /// - Provides a reasonable starting point for TallyMaps that will receive merged results
    pub const fn initial_reduce_tally_map_capacity(&self, chunk_size_in_lines: usize) -> usize {
        // Start with a base capacity derived from chunk size
        // Division by 10 represents rough estimate of unique words from multiple chunks
        let base_capacity = chunk_size_in_lines / UNIQUENESS_RATIO as usize;
        // Ensure a reasonable minimum capacity
        if base_capacity < 64 {
            64
        } else {
            base_capacity
        }
    }

    /// Calculate capacity for a line buffer
    pub const fn line_buffer_capacity(&self) -> usize {
        CHARS_PER_LINE
    }

    /// Calculate capacity for content buffer based on size hint
    ///
    /// - With size hint: Exact size in bytes
    /// - Without size hint: Default chunk size in bytes
    pub const fn content_buffer_capacity(&self) -> usize {
        match self.size_hint {
            SizeHint::None => self.chunk_size,
            SizeHint::Bytes(size) => size,
        }
    }

    /// Initialize the Rayon thread pool with configuration from `Performance`
    pub fn init_thread_pool(&self) {
        init_thread_pool(self.threads, self.verbose)
    }

    /// The number of estimated lines in the chunk
    pub fn estimated_lines_per_chunk(&self) -> usize {
        (self.chunk_size() / CHARS_PER_LINE).max(128)
    }

    /// Calculate per-thread capacity for parallel buffered processing
    ///
    /// Given a buffered input with known line count, estimates the capacity
    /// for each thread's `TallyMap` based on:
    /// - Number of lines in the buffer
    /// - Average bytes per line
    /// - Number of threads in the pool
    pub fn tally_map_capacity_for_buffered_lines(
        &self,
        content_length: usize,
        lines_count: usize,
    ) -> usize {
        if lines_count == 0 {
            return self.default_capacity;
        }

        let avg_line_length = content_length / lines_count;

        // Estimate lines per thread
        let num_threads = self.estimate_thread_pool_size();
        let lines_per_thread = lines_count.div_ceil(num_threads);

        self.tally_map_capacity_for_content(lines_per_thread * avg_line_length)
    }
}

impl fmt::Display for Performance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Performance {{ threads: {}, chunk_size: {} }}",
            self.threads, self.chunk_size
        )
    }
}
