//! A tally of words with a count of the number of times each appears.
//!
//! A `WordTally` represents a tally of the total number of times each word
//! appears in an input source that implements `Read`. When a `WordTally` is
//! constructed, the provided input is iterated over line by line to count words.
//! Ordered pairs of words and their count are stored in the `tally` field.
//!
//! The `unicode-segmentation` Crate segments along "Word Bounaries" according
//! to the [Unicode Standard Annex #29](http://www.unicode.org/reports/tr29/).
//!
//! # `Case`, `Sort` and `Filters`

//! In addition to source input, a `WordTally` is constructed with options for
//! `Case` normalization, `Sort` order and word `Filters`. `Case` options include
//! `Original` (case sensitive) and `Lower` or `Upper` case normalization. `Sort`
//! order can be `Unsorted` or sorted `Desc` (descending) or `Asc` (ascending).
//! A `tally` can be sorted at construction and resorted with the `sort` method.
//! Sorting doesn't impact the `count` or `uniq_count` fields. `Filter`s can
//! be used to provide list of words that should or shouldn't be tallied.
//!
//! # Examples
//!
//! ```
//! use word_tally::{Filters, Options, WordTally};
//!
//! let input = "Cinquedea".as_bytes();
//! let words = WordTally::new(input, Options::default(), Filters::default());
//! let expected_tally: Box<[(Box<str>, usize)]> = [("cinquedea".into(), 1)].into();
//!
//! assert_eq!(words.into_tally(), expected_tally);
//! ```
use indexmap::IndexMap;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Read};
use unicode_segmentation::UnicodeSegmentation;

pub mod filters;
pub mod input;
pub mod options;

pub use filters::{ExcludeWords, Filters, MinChars, MinCount};
pub use input::Input;
pub use options::{Case, Options, Sort};

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
const ENV_WORD_DENSITY: &str = "WORD_TALLY_UNIQUE_WORD_DENSITY";
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

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub struct WordTally {
    /// Ordered pairs of words and the count of times they appear.
    tally: Box<[(Box<str>, usize)]>,

    /// Word tallying options like case normalization and sort order.
    options: Options,

    /// Filters that limit words from being tallied.
    filters: Filters,

    /// The sum of all words tallied.
    count: usize,

    /// The sum of uniq words tallied.
    uniq_count: usize,

    /// Configuration for word tallying and processing (not serialized)
    #[serde(skip)]
    config: Config,
}

/// A `tally` supports `iter` and can also be represented as a `Vec`.
impl From<WordTally> for Vec<(Box<str>, usize)> {
    fn from(word_tally: WordTally) -> Self {
        word_tally.into_tally().into_vec()
    }
}

/// A `tally` can also be iterated over directly from a `WordTally`.
impl<'a> IntoIterator for &'a WordTally {
    type Item = &'a (Box<str>, usize);
    type IntoIter = std::slice::Iter<'a, (Box<str>, usize)>;
    fn into_iter(self) -> Self::IntoIter {
        self.tally.iter()
    }
}

/// `WordTally` fields are eagerly populated upon construction and exposed by getter methods.
impl WordTally {
    /// Constructs a new `WordTally` from a source that implements `Read` like file or stdin.
    /// Uses sequential processing by default.
    pub fn new<T: Read>(input: T, options: Options, filters: Filters) -> Self {
        Self::new_with_size(input, options, filters, None)
    }

    /// Constructs a new `WordTally` from a source using default options and filters.
    /// Uses sequential processing by default.
    pub fn new_with_defaults<T: Read>(input: T) -> Self {
        Self::new(input, Options::default(), Filters::default())
    }

    /// Constructs a new `WordTally` with an optional size hint for better capacity estimation.
    pub fn new_with_size<T: Read>(
        input: T,
        options: Options,
        filters: Filters,
        size_hint: Option<u64>,
    ) -> Self {
        Self::new_with_config(input, options, filters, size_hint, Config::from_env())
    }

    /// Constructs a new `WordTally` using parallel processing
    pub fn new_parallel<T: Read>(input: T, options: Options, filters: Filters) -> Self {
        Self::new_parallel_with_size(input, options, filters, None)
    }

    /// Constructs a new `WordTally` using parallel processing with default options and filters.
    pub fn new_parallel_with_defaults<T: Read>(input: T) -> Self {
        Self::new_parallel(input, Options::default(), Filters::default())
    }

    /// Constructs a new `WordTally` using parallel processing with an optional size hint.
    pub fn new_parallel_with_size<T: Read>(
        input: T,
        options: Options,
        filters: Filters,
        size_hint: Option<u64>,
    ) -> Self {
        Self::new_parallel_with_config(input, options, filters, size_hint, Config::from_env())
    }

    /// Constructs a new `WordTally` with custom configuration
    pub fn new_with_config<T: Read>(
        input: T,
        options: Options,
        filters: Filters,
        size_hint: Option<u64>,
        config: Config,
    ) -> Self {
        let reader = BufReader::new(input);
        let mut instance = Self {
            options,
            filters,
            config,
            ..Default::default()
        };

        let mut tally_map = instance.tally_map(reader, options.case, size_hint);
        instance.filters.apply(&mut tally_map, options.case);

        let count = tally_map.values().sum();
        let tally: Box<[_]> = tally_map.into_iter().collect();
        let uniq_count = tally.len();

        instance.tally = tally;
        instance.count = count;
        instance.uniq_count = uniq_count;
        instance.sort(options.sort);

        instance
    }

    /// Constructs a new `WordTally` with parallel processing and custom configuration
    pub fn new_parallel_with_config<T: Read>(
        input: T,
        options: Options,
        filters: Filters,
        size_hint: Option<u64>,
        config: Config,
    ) -> Self {
        Self::init_thread_pool();

        let reader = BufReader::new(input);
        let mut instance = Self {
            options,
            filters,
            config,
            ..Default::default()
        };

        let mut tally_map = instance.tally_map_parallel(
            reader,
            instance.config.chunk_size(),
            options.case,
            size_hint,
        );
        instance.filters.apply(&mut tally_map, options.case);

        let count = tally_map.values().sum();
        let tally: Box<[_]> = tally_map.into_iter().collect();
        let uniq_count = tally.len();

        instance.tally = tally;
        instance.count = count;
        instance.uniq_count = uniq_count;
        instance.sort(options.sort);

        instance
    }

    /// Sorts the `tally` field in place if a sort order other than `Unsorted` is provided.
    pub fn sort(&mut self, sort: Sort) {
        sort.apply(self);
    }

    /// Gets the `tally` field.
    pub const fn tally(&self) -> &[(Box<str>, usize)] {
        &self.tally
    }

    /// Consumes the `tally` field.
    pub fn into_tally(self) -> Box<[(Box<str>, usize)]> {
        self.tally
    }

    /// Gets the `options` field.
    pub const fn options(&self) -> Options {
        self.options
    }

    /// Gets the `filters` field.
    pub const fn filters(&self) -> &Filters {
        &self.filters
    }

    /// Gets the `uniq_count` field.
    pub const fn uniq_count(&self) -> usize {
        self.uniq_count
    }

    /// Gets the `count` field.
    pub const fn count(&self) -> usize {
        self.count
    }

    /// Estimates the capacity for word maps based on input size
    fn estimate_capacity(&self, size_hint: Option<u64>) -> usize {
        self.config.estimate_capacity(size_hint)
    }

    /// Estimates the capacity for per-chunk word maps based on chunk size
    const fn estimate_chunk_capacity(&self, chunk_size: usize) -> usize {
        self.config.estimate_chunk_capacity(chunk_size)
    }

    /// Sequential implementation for word tallying
    fn tally_map<T: Read>(
        &self,
        reader: BufReader<T>,
        case: Case,
        size_hint: Option<u64>,
    ) -> IndexMap<Box<str>, usize> {
        let estimated_capacity = self.estimate_capacity(size_hint);
        let mut tally = IndexMap::with_capacity(estimated_capacity);
        for line in reader.lines().map_while(Result::ok) {
            for word in line.unicode_words() {
                *tally.entry(case.normalize(word)).or_insert(0) += 1;
            }
        }
        tally
    }

    /// Initializes the rayon thread pool with configuration from environment
    fn init_thread_pool() {
        static INIT_THREAD_POOL: std::sync::Once = std::sync::Once::new();
        INIT_THREAD_POOL.call_once(|| {
            if let Ok(thread_count) = std::env::var("WORD_TALLY_THREADS") {
                if let Ok(num_threads) = thread_count.parse::<usize>() {
                    if let Err(e) = rayon::ThreadPoolBuilder::new()
                        .num_threads(num_threads)
                        .build_global()
                    {
                        eprintln!("Warning: Failed to set thread pool size: {}", e);
                    }
                }
            }
        });
    }

    /// Parallel implementation for larger inputs with optimized chunking strategy
    fn tally_map_parallel<T: Read>(
        &self,
        reader: BufReader<T>,
        chunk_size: u32,
        case: Case,
        size_hint: Option<u64>,
    ) -> IndexMap<Box<str>, usize> {
        let estimated_capacity = self.estimate_capacity(size_hint);
        let num_threads = rayon::current_num_threads();
        let mut result_map = IndexMap::with_capacity(estimated_capacity);
        let mut lines_batch = Vec::with_capacity(chunk_size as usize);

        for line in reader.lines().map_while(Result::ok) {
            lines_batch.push(line);

            if lines_batch.len() >= chunk_size as usize {
                self.process_and_merge_batch(
                    &mut result_map,
                    &lines_batch,
                    case,
                    estimated_capacity,
                    num_threads,
                );
                lines_batch.clear();
            }
        }

        self.process_and_merge_batch(
            &mut result_map,
            &lines_batch,
            case,
            estimated_capacity,
            num_threads,
        );

        result_map
    }

    /// Processes a batch and merges the results
    #[inline]
    fn process_and_merge_batch(
        &self,
        result_map: &mut IndexMap<Box<str>, usize>,
        lines: &[String],
        case: Case,
        estimated_capacity: usize,
        num_threads: usize,
    ) {
        if lines.is_empty() {
            return;
        }
        let batch_map = self.process_batch(lines, case, estimated_capacity, num_threads);
        Self::merge_map_into(result_map, batch_map);
    }

    /// Merges maps by combining word counts
    #[inline]
    fn merge_map_into(dest: &mut IndexMap<Box<str>, usize>, source: IndexMap<Box<str>, usize>) {
        for (word, count) in source {
            *dest.entry(word).or_insert(0) += count;
        }
    }

    /// Processes a batch of lines in parallel
    fn process_batch(
        &self,
        lines: &[String],
        case: Case,
        estimated_capacity: usize,
        num_threads: usize,
    ) -> IndexMap<Box<str>, usize> {
        let chunk_size = std::cmp::max(4, lines.len() / num_threads.max(1));
        let thread_maps: Vec<IndexMap<Box<str>, usize>> = lines
            .par_chunks(chunk_size)
            .map(|chunk| {
                let mut local_counts =
                    IndexMap::with_capacity(self.estimate_chunk_capacity(chunk.len()));
                for line in chunk {
                    for word in line.unicode_words() {
                        *local_counts.entry(case.normalize(word)).or_insert(0) += 1;
                    }
                }
                local_counts
            })
            .collect();

        let mut result = IndexMap::with_capacity(estimated_capacity);
        for map in thread_maps {
            Self::merge_map_into(&mut result, map);
        }

        result
    }
}
