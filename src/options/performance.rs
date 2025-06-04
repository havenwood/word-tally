//! Configuration for word tallying performance.

use super::threads::Threads;
use core::fmt::{self, Display, Formatter};
use serde::{Deserialize, Serialize};
use std::env;
use std::str::FromStr;
use std::sync::OnceLock;

/// Performance tuning configuration parameters.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Performance {
    /// Ratio used to estimate number of unique words based on input size.
    pub uniqueness_ratio: u16,

    /// Words-per-KB of text.
    pub words_per_kb: u16,

    /// Size of chunks for parallel processing (in bytes).
    pub chunk_size: u64,

    /// Base stdin size for unknown-sized inputs.
    pub base_stdin_size: u64,

    /// Thread configuration for parallel processing.
    pub threads: Threads,
}

impl Default for Performance {
    fn default() -> Self {
        Self {
            uniqueness_ratio: Self::UNIQUENESS_RATIO,
            words_per_kb: Self::WORDS_PER_KB,
            chunk_size: Self::PAR_CHUNK_SIZE,
            base_stdin_size: Self::BASE_STDIN_SIZE,
            threads: Threads::default(),
        }
    }
}

impl Performance {
    /// Safely convert u64 to usize for capacity calculations.
    /// Returns `usize::MAX` if the value would overflow.
    #[inline]
    #[allow(clippy::cast_possible_truncation)]
    const fn saturating_cast(value: u64) -> usize {
        if value > usize::MAX as u64 {
            usize::MAX
        } else {
            value as usize
        }
    }

    /// Estimated is 128 words per KB.
    const WORDS_PER_KB: u16 = 128;
    /// At most, 512 words per KB (one word every other byte).
    const MAX_WORDS_PER_KB: u16 = 512;
    /// Estimated 32 words per unique word (32:1 ratio).
    const UNIQUENESS_RATIO: u16 = 32;
    /// Estimated stdin size is 256KB.
    const BASE_STDIN_SIZE: u64 = 256 * 1024;
    /// Target chunks per thread (2-4 chunks per thread helps with work stealing).
    pub const PAR_CHUNKS_PER_THREAD: u8 = 4;
    /// Chunk size for parallel processing is 64KB.
    pub const PAR_CHUNK_SIZE: u64 = 64 * 1024;

    // Environment variable names for configuration.
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
                chunk_size: Self::parse_env_var(Self::ENV_CHUNK_SIZE, Self::PAR_CHUNK_SIZE),
                base_stdin_size,
                threads: Self::parse_threads(),
            }
        })
    }

    /// Set the base stdin size for unknown-sized inputs.
    #[must_use]
    pub const fn with_base_stdin_size(mut self, size: u64) -> Self {
        self.base_stdin_size = size;
        self
    }

    /// Set the chunk size for this configuration.
    #[must_use]
    pub const fn with_chunk_size(mut self, size: u64) -> Self {
        self.chunk_size = size;
        self
    }

    /// Set the thread count.
    #[must_use]
    pub const fn with_threads(mut self, threads: Threads) -> Self {
        self.threads = threads;
        self
    }

    /// Set the uniqueness ratio for this configuration.
    #[must_use]
    pub const fn with_uniqueness_ratio(mut self, ratio: u16) -> Self {
        self.uniqueness_ratio = ratio;
        self
    }

    /// Set the words-per-KB for this configuration.
    #[must_use]
    pub const fn with_words_per_kb(mut self, words_per_kb: u16) -> Self {
        self.words_per_kb = if words_per_kb > Self::MAX_WORDS_PER_KB {
            Self::MAX_WORDS_PER_KB
        } else {
            words_per_kb
        };
        self
    }

    /// Get the base stdin size.
    #[must_use]
    pub const fn base_stdin_size(&self) -> u64 {
        self.base_stdin_size
    }

    /// Get the base stdin size as usize.
    #[must_use]
    pub const fn base_stdin_size_usize(&self) -> usize {
        Self::saturating_cast(self.base_stdin_size)
    }

    /// Get the chunk size.
    #[must_use]
    pub const fn chunk_size(&self) -> u64 {
        self.chunk_size
    }

    /// Get the thread configuration.
    #[must_use]
    pub const fn threads(&self) -> Threads {
        self.threads
    }

    /// Calculate capacity based on input size in bytes.
    #[must_use]
    pub const fn chunk_capacity(&self, byte_size: u64) -> usize {
        let kb_size = byte_size / 1024;
        let estimated_words = kb_size.saturating_mul(self.words_per_kb as u64);
        let capacity = estimated_words / self.uniqueness_ratio as u64;

        Self::saturating_cast(capacity)
    }

    /// Estimated capacity based on input size.
    #[must_use]
    pub const fn capacity(&self, input_size: Option<u64>) -> usize {
        match input_size {
            None => Self::base_stdin_tally_capacity(),
            Some(size) => self.chunk_capacity(size),
        }
    }

    /// Default configuration value for export.
    #[must_use]
    pub const fn base_stdin_tally_capacity() -> usize {
        let capacity = (Self::BASE_STDIN_SIZE / 1024).saturating_mul(Self::WORDS_PER_KB as u64)
            / Self::UNIQUENESS_RATIO as u64;
        Self::saturating_cast(capacity)
    }

    /// Calculate optimal number of chunks based on content length.
    #[must_use]
    pub const fn total_chunks(&self, content_len: u64) -> u64 {
        if content_len == 0 {
            return 0;
        }

        content_len.div_ceil(self.chunk_size)
    }

    /// Get the batch buffer size for streaming operations.
    #[must_use]
    pub fn stream_batch_size() -> u64 {
        let thread_count = rayon::current_num_threads() as u64;
        u64::from(Self::PAR_CHUNKS_PER_THREAD)
            .saturating_mul(thread_count)
            .saturating_mul(Self::PAR_CHUNK_SIZE)
    }

    /// Calculate total number of chunks for streaming based on thread count.
    #[must_use]
    pub fn stream_total_chunks() -> usize {
        rayon::current_num_threads() * Self::PAR_CHUNKS_PER_THREAD as usize
    }

    /// Calculate streaming batch size based on input size.
    #[must_use]
    pub fn stream_batch_size_for_input(&self, input_size: u64) -> u64 {
        let total_chunks = Self::stream_total_chunks() as u64;
        let target_batch_size = input_size.div_ceil(total_chunks);
        target_batch_size.max(Self::PAR_CHUNK_SIZE)
    }

    /// Calculate chunk boundary capacity based on total chunks.
    #[must_use]
    pub const fn chunk_boundary_capacity(total_chunks: u64) -> usize {
        Self::saturating_cast(total_chunks.saturating_add(1))
    }

    /// Calculate chunk boundary capacity for streaming.
    #[must_use]
    pub fn stream_boundary_capacity() -> usize {
        Self::chunk_boundary_capacity(Self::stream_total_chunks() as u64)
    }

    /// Calculate chunk size for streaming based on target batch size.
    #[must_use]
    pub const fn stream_chunk_size(target_batch_size: usize) -> usize {
        target_batch_size / Self::PAR_CHUNKS_PER_THREAD as usize
    }

    // Env-parsing helpers

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
            "Performance {{ tally_capacity: {}, uniqueness: {}, words/kb: {}, chunk: {}, stdin_size: {}, threads: {} }}",
            Self::base_stdin_tally_capacity(),
            self.uniqueness_ratio,
            self.words_per_kb,
            self.chunk_size(),
            self.base_stdin_size(),
            self.threads()
        )
    }
}
