//! Configuration for word tallying and processing.

use serde::{Deserialize, Serialize};

/// Configuration for word tallying and processing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Config {
    /// Default capacity for IndexMap when no size hint is available
    default_capacity: usize,
    /// Ratio used to estimate number of unique words based on input size
    uniqueness_ratio: u8,
    /// Estimated number of unique words per character in input
    unique_word_density: u8,
    /// Size of chunks for parallel processing (in bytes)
    chunk_size: u32,
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

impl Default for Config {
    fn default() -> Self {
        Self {
            default_capacity: DEFAULT_CAPACITY,
            uniqueness_ratio: DEFAULT_UNIQUENESS_RATIO,
            unique_word_density: DEFAULT_WORD_DENSITY,
            chunk_size: DEFAULT_CHUNK_SIZE,
        }
    }
}

impl Config {
    /// Create a new configuration for a word tally
    pub const fn new(
        default_capacity: usize,
        uniqueness_ratio: u8,
        unique_word_density: u8,
        chunk_size: u32,
    ) -> Self {
        Self {
            default_capacity,
            uniqueness_ratio,
            unique_word_density,
            chunk_size,
        }
    }

    /// Create configuration from environment variables if present
    pub fn from_env() -> Self {
        use std::sync::OnceLock;

        // Parse environment variables only once and cache the result
        static CONFIG: OnceLock<Config> = OnceLock::new();

        *CONFIG.get_or_init(|| {
            fn parse_env_var<T: std::str::FromStr>(name: &str, default: T) -> T {
                std::env::var(name)
                    .ok()
                    .and_then(|value| value.parse().ok())
                    .unwrap_or(default)
            }

            Self {
                default_capacity: parse_env_var(ENV_DEFAULT_CAPACITY, DEFAULT_CAPACITY),
                uniqueness_ratio: parse_env_var(ENV_UNIQUENESS_RATIO, DEFAULT_UNIQUENESS_RATIO),
                unique_word_density: parse_env_var(ENV_WORD_DENSITY, DEFAULT_WORD_DENSITY),
                chunk_size: parse_env_var(ENV_CHUNK_SIZE, DEFAULT_CHUNK_SIZE),
            }
        })
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

    /// Estimate map capacity based on input size
    pub fn estimate_capacity(&self, size_hint: Option<u64>) -> usize {
        size_hint.map_or(self.default_capacity, |size| {
            (size / self.uniqueness_ratio as u64) as usize
        })
    }

    /// Estimate chunk map capacity based on chunk size
    pub const fn estimate_chunk_capacity(&self, chunk_size: usize) -> usize {
        chunk_size * self.unique_word_density as usize
    }
}