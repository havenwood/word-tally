//! Configuration for word tallying performance.

use super::threads::Threads;
use core::fmt::{self, Display, Formatter};
use serde::{Deserialize, Serialize};
use std::env;
use std::str::FromStr;
use std::sync::OnceLock;

/// Performance tuning configuration parameters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Performance {
    /// Ratio used to estimate number of unique words based on input size.
    pub uniqueness_ratio: u16,

    /// Words-per-KB of text.
    pub words_per_kb: u16,

    /// Size of chunks for parallel processing (in bytes).
    pub chunk_size: usize,

    /// Base stdin size for unknown-sized inputs.
    pub base_stdin_size: usize,

    /// Thread configuration for parallel processing.
    pub threads: Threads,

    /// Option to print verbose messages.
    pub verbose: bool,
}

impl Default for Performance {
    fn default() -> Self {
        Self {
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
    /// Default configuration constants.
    const WORDS_PER_KB: u16 = 128; // Words-per-KB estimation
    const MAX_WORDS_PER_KB: u16 = 512; // Maximum words-per-KB (one word every other byte)
    const UNIQUENESS_RATIO: u16 = 256; // 256:1 ratio saves memory (~10:1 is more reasonable for books)
    const CHUNK_SIZE: usize = 64 * 1024; // 64KB
    const BASE_STDIN_SIZE: usize = 256 * 1024; // 256KB estimated stdin size
    const MIN_THREAD_CAPACITY: usize = 1024; // Minimum capacity per thread
    const CHARS_PER_LINE: usize = 80; // Chars-per-line estimation

    /// Environment variable names for configuration.
    const ENV_STDIN_BUFFER_SIZE: &str = "WORD_TALLY_STDIN_BUFFER_SIZE";
    const ENV_CHUNK_SIZE: &str = "WORD_TALLY_CHUNK_SIZE";
    const ENV_UNIQUENESS_RATIO: &str = "WORD_TALLY_UNIQUENESS_RATIO";
    const ENV_WORDS_PER_KB: &str = "WORD_TALLY_WORDS_PER_KB";
    const ENV_THREADS: &str = "WORD_TALLY_THREADS";

    /// Create performance configuration from environment variables if present.
    pub fn from_env() -> Self {
        // Parse environment variables only once and cache the result
        static CONFIG: OnceLock<Performance> = OnceLock::new();

        *CONFIG.get_or_init(|| {
            let base_stdin_size =
                Self::parse_env_var(Self::ENV_STDIN_BUFFER_SIZE, Self::BASE_STDIN_SIZE);

            Self {
                uniqueness_ratio: Self::parse_env_var(
                    Self::ENV_UNIQUENESS_RATIO,
                    Self::UNIQUENESS_RATIO,
                ),
                words_per_kb: Self::parse_env_var(Self::ENV_WORDS_PER_KB, Self::WORDS_PER_KB)
                    .min(Self::MAX_WORDS_PER_KB),
                chunk_size: Self::parse_env_var(Self::ENV_CHUNK_SIZE, Self::CHUNK_SIZE),
                base_stdin_size,
                threads: Self::parse_threads(),
                verbose: false,
            }
        })
    }

    /// Set the base stdin size for unknown-sized inputs.
    pub const fn with_base_stdin_size(mut self, size: usize) -> Self {
        self.base_stdin_size = size;
        self
    }

    /// Set the chunk size for this configuration.
    pub const fn with_chunk_size(mut self, size: usize) -> Self {
        self.chunk_size = size;
        self
    }

    /// Set the thread count.
    pub const fn with_threads(mut self, threads: Threads) -> Self {
        self.threads = threads;
        self
    }

    /// Set the uniqueness ratio for this configuration.
    pub const fn with_uniqueness_ratio(mut self, ratio: u16) -> Self {
        self.uniqueness_ratio = ratio;
        self
    }

    /// Set verbose mode.
    pub const fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Set the words-per-KB for this configuration.
    pub const fn with_words_per_kb(mut self, words_per_kb: u16) -> Self {
        self.words_per_kb = if words_per_kb > Self::MAX_WORDS_PER_KB {
            Self::MAX_WORDS_PER_KB
        } else {
            words_per_kb
        };
        self
    }

    /// Get the base stdin size.
    pub const fn base_stdin_size(&self) -> usize {
        self.base_stdin_size
    }

    /// Get the chunk size.
    pub const fn chunk_size(&self) -> usize {
        self.chunk_size
    }

    /// Get the thread configuration.
    pub const fn threads(&self) -> Threads {
        self.threads
    }

    /// Calculate capacity based on input size in bytes.
    const fn calculate_capacity(&self, size_bytes: usize) -> usize {
        Self::calculate_capacity_static(size_bytes, self.words_per_kb, self.uniqueness_ratio)
    }

    /// Static version of capacity calculation for use in const contexts.
    const fn calculate_capacity_static(
        size_bytes: usize,
        words_per_kb: u16,
        uniqueness_ratio: u16,
    ) -> usize {
        let kb_size = size_bytes / 1024;
        let estimated_words = kb_size * words_per_kb as usize;
        estimated_words / uniqueness_ratio as usize
    }

    /// Estimated capacity based on input size.
    pub const fn capacity(&self, input_size: Option<usize>) -> usize {
        match input_size {
            None => Self::base_stdin_tally_capacity(),
            Some(size) => self.calculate_capacity(size),
        }
    }

    /// Default configuration value for export.
    pub const fn base_stdin_tally_capacity() -> usize {
        Self::BASE_STDIN_SIZE / 1024 * Self::WORDS_PER_KB as usize / Self::UNIQUENESS_RATIO as usize
    }

    /// Estimated lines per chunk based on chunk size and average line length.
    pub const fn lines_per_chunk(&self) -> usize {
        let lines = self.chunk_size / Self::CHARS_PER_LINE;
        if lines > 128 { lines } else { 128 }
    }

    /// Capacity for each thread in parallel processing.
    pub fn capacity_per_thread(&self) -> usize {
        let thread_count = self.threads.count();
        let per_thread = Self::base_stdin_tally_capacity() / thread_count;

        // Give each thread a reasonable minimum capacity, but don't exceed total
        per_thread
            .max(Self::MIN_THREAD_CAPACITY)
            .min(Self::base_stdin_tally_capacity())
    }

    //
    // Env-parsing helpers
    //

    /// Parse numeric environment variable with fallback to default value.
    fn parse_env_var<T: FromStr>(name: &str, default: T) -> T {
        env::var(name)
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(default)
    }

    /// Parse thread count from `WORD_TALLY_THREADS` environment variable.
    fn parse_threads() -> Threads {
        env::var(Self::ENV_THREADS)
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

impl Display for Performance {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Performance {{ tally_capacity: {}, uniqueness: {}, words/kb: {}, chunk: {}, stdin_size: {}, threads: {}, verbose: {} }}",
            Self::base_stdin_tally_capacity(),
            self.uniqueness_ratio,
            self.words_per_kb,
            self.chunk_size(),
            self.base_stdin_size(),
            self.threads(),
            self.verbose
        )
    }
}
