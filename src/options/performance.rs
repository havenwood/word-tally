//! Configuration for word tallying performance.

use super::threads::Threads;
use core::fmt;
use serde::{Deserialize, Serialize};

/// # Performance tuning configuration parameters
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Performance {
    /// Base stdin capacity for `TallyMap` (`IndexMap`) when no size hint is available
    pub base_stdin_tally_capacity: usize,

    /// Ratio used to estimate number of unique words based on input size
    pub uniqueness_ratio: u8,

    /// Words-per-KB of text
    pub words_per_kb: u8,

    /// Size of chunks for parallel processing (in bytes)
    pub chunk_size: usize,

    /// Base stdin size for unknown-sized inputs
    pub base_stdin_size: usize,

    /// Thread configuration for parallel processing
    pub threads: Threads,

    /// Option to print verbose messages
    pub verbose: bool,
}

/// Default configuration value for export
pub const BASE_STDIN_TALLY_CAPACITY: usize = 5120; // ~5k unique words

impl Default for Performance {
    fn default() -> Self {
        Self {
            base_stdin_tally_capacity: BASE_STDIN_TALLY_CAPACITY,
            uniqueness_ratio: Self::UNIQUENESS_RATIO,
            words_per_kb: Self::WORDS_PER_KB,
            chunk_size: Self::CHUNK_SIZE,
            base_stdin_size: Self::BASE_STDIN_SIZE,
            threads: Threads::default(),
            verbose: false,
        }
    }
}

impl Performance {
    /// Default configuration constants
    const WORDS_PER_KB: u8 = 200; // Words-per-KB estimation
    const UNIQUENESS_RATIO: u8 = 10; // 10:1 ratio
    const CHUNK_SIZE: usize = 64 * 1024; // 64KB
    const BASE_STDIN_SIZE: usize = 256 * 1024; // 256KB estimated stdin size
    const MIN_THREAD_CAPACITY: usize = 1024; // Minimum capacity per thread
    const CHARS_PER_LINE: usize = 80; // Chars-per-line estimation

    /// Environment variable names for configuration
    const ENV_STDIN_BUFFER_SIZE: &str = "WORD_TALLY_STDIN_BUFFER_SIZE";
    const ENV_CHUNK_SIZE: &str = "WORD_TALLY_CHUNK_SIZE";
    const ENV_UNIQUENESS_RATIO: &str = "WORD_TALLY_UNIQUENESS_RATIO";
    const ENV_WORDS_PER_KB: &str = "WORD_TALLY_WORDS_PER_KB";
    const ENV_VERBOSE: &str = "WORD_TALLY_VERBOSE";
    const ENV_THREADS: &str = "WORD_TALLY_THREADS";

    /// Create performance configuration from environment variables if present
    pub fn from_env() -> Self {
        use std::sync::OnceLock;

        // Parse environment variables only once and cache the result
        static CONFIG: OnceLock<Performance> = OnceLock::new();

        *CONFIG.get_or_init(|| {
            let base_stdin_size =
                Self::parse_env_var(Self::ENV_STDIN_BUFFER_SIZE, Self::BASE_STDIN_SIZE);
            let stdin_capacity = Self::calculate_capacity_static(
                base_stdin_size,
                Self::WORDS_PER_KB,
                Self::UNIQUENESS_RATIO,
            );

            Self {
                base_stdin_tally_capacity: stdin_capacity,
                uniqueness_ratio: Self::parse_env_var(
                    Self::ENV_UNIQUENESS_RATIO,
                    Self::UNIQUENESS_RATIO,
                ),
                words_per_kb: Self::parse_env_var(Self::ENV_WORDS_PER_KB, Self::WORDS_PER_KB),
                chunk_size: Self::parse_env_var(Self::ENV_CHUNK_SIZE, Self::CHUNK_SIZE),
                base_stdin_size,
                threads: Self::parse_threads(),
                verbose: Self::parse_verbose(),
            }
        })
    }

    /// Set the base stdin size for unknown-sized inputs
    pub const fn with_base_stdin_size(mut self, size: usize) -> Self {
        self.base_stdin_size = size;
        self
    }

    /// Create a new configuration with custom base stdin tally capacity setting
    pub const fn with_base_stdin_tally_capacity(mut self, capacity: usize) -> Self {
        self.base_stdin_tally_capacity = capacity;
        self
    }

    /// Set the chunk size for this configuration
    pub const fn with_chunk_size(mut self, size: usize) -> Self {
        self.chunk_size = size;
        self
    }

    /// Set the thread count
    pub const fn with_threads(mut self, threads: Threads) -> Self {
        self.threads = threads;
        self
    }

    /// Set the uniqueness ratio for this configuration
    pub const fn with_uniqueness_ratio(mut self, ratio: u8) -> Self {
        self.uniqueness_ratio = ratio;
        self
    }

    /// Set verbose mode
    pub const fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Set the words-per-KB for this configuration
    pub const fn with_words_per_kb(mut self, words_per_kb: u8) -> Self {
        self.words_per_kb = words_per_kb;
        self
    }

    /// Calculate capacity based on input size in bytes
    const fn calculate_capacity(&self, size_bytes: usize) -> usize {
        Self::calculate_capacity_static(size_bytes, self.words_per_kb, self.uniqueness_ratio)
    }

    /// Static version of capacity calculation for use in const contexts
    const fn calculate_capacity_static(
        size_bytes: usize,
        words_per_kb: u8,
        uniqueness_ratio: u8,
    ) -> usize {
        let kb_size = size_bytes / 1024;
        let estimated_words = kb_size * words_per_kb as usize;
        estimated_words / uniqueness_ratio as usize
    }

    /// Estimated capacity based on input size
    pub const fn capacity(&self, input_size: Option<usize>) -> usize {
        match input_size {
            None => self.base_stdin_tally_capacity,
            Some(size) => self.calculate_capacity(size),
        }
    }

    /// Estimated lines per chunk based on chunk size and average line length
    pub const fn lines_per_chunk(&self) -> usize {
        let lines = self.chunk_size / Self::CHARS_PER_LINE;
        if lines > 128 { lines } else { 128 }
    }

    /// Capacity for each thread in parallel processing
    pub fn capacity_per_thread(&self) -> usize {
        let thread_count = self.threads.count();
        let per_thread = self.base_stdin_tally_capacity / thread_count;

        // Give each thread a reasonable minimum capacity, but don't exceed total
        per_thread
            .max(Self::MIN_THREAD_CAPACITY)
            .min(self.base_stdin_tally_capacity)
    }

    //
    // Env-parsing helpers
    //

    /// Parse numeric environment variable with fallback to default value
    fn parse_env_var<T: std::str::FromStr>(name: &str, default: T) -> T {
        std::env::var(name)
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(default)
    }

    /// Parse verbose flag from WORD_TALLY_VERBOSE environment variable
    /// Returns true for "1", "true", "yes", or "on"
    fn parse_verbose() -> bool {
        matches!(
            std::env::var(Self::ENV_VERBOSE).ok().as_deref(),
            Some("1" | "true" | "yes" | "on")
        )
    }

    /// Parse thread count from `WORD_TALLY_THREADS` environment variable
    fn parse_threads() -> Threads {
        std::env::var(Self::ENV_THREADS)
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
}

impl fmt::Display for Performance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Performance {{ tally_capacity: {}, uniqueness: {}, words/kb: {}, chunk: {}, stdin_size: {}, threads: {}, verbose: {} }}",
            self.base_stdin_tally_capacity,
            self.uniqueness_ratio,
            self.words_per_kb,
            self.chunk_size,
            self.base_stdin_size,
            self.threads,
            self.verbose
        )
    }
}
