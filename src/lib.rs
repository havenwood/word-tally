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
//! use word_tally::{Config, Filters, Options, WordTally};
//!
//! let input = "Cinquedea".as_bytes();
//! let words = WordTally::new(input, Options::default(), Filters::default(), Config::default());
//! let expected_tally: Box<[(Box<str>, usize)]> = [("cinquedea".into(), 1)].into();
//!
//! assert_eq!(words.into_tally(), expected_tally);
//! ```
use indexmap::IndexMap;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Read};
use unicode_segmentation::UnicodeSegmentation;

pub mod config;
pub mod filters;
pub mod input;
pub mod options;
pub mod output;

pub use config::{Concurrency, Config, SizeHint, Threads};
pub use filters::{ExcludePatterns, ExcludeWords, Filters, IncludePatterns, MinChars, MinCount};
pub use input::Input;
pub use options::{Case, Format, Options, Sort};
pub use output::Output;

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
    /// Constructs a new `WordTally` from a source that implements `Read`.
    ///
    /// Takes options, filters, and a config.
    pub fn new<T: Read>(input: T, options: Options, filters: Filters, config: Config) -> Self {
        // Initialize thread pool if using parallel processing
        if matches!(config.concurrency(), Concurrency::Parallel) {
            Self::init_thread_pool(config.threads());
        }

        let reader = BufReader::new(input);
        let mut instance = Self {
            options,
            filters,
            config,
            ..Default::default()
        };

        // Choose processing method based on concurrency setting
        let mut tally_map = match instance.config.concurrency() {
            Concurrency::Sequential => instance.tally_map(reader, options.case),
            Concurrency::Parallel => {
                instance.tally_map_parallel(reader, instance.config.chunk_size(), options.case)
            }
        };

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
    const fn estimate_capacity(&self) -> usize {
        self.config.estimate_capacity()
    }

    /// Estimates the capacity for per-chunk word maps based on chunk size
    const fn estimate_chunk_capacity(&self, chunk_size: usize) -> usize {
        self.config.estimate_chunk_capacity(chunk_size)
    }

    /// Sequential implementation for word tallying
    fn tally_map<T: Read>(&self, reader: BufReader<T>, case: Case) -> IndexMap<Box<str>, usize> {
        let estimated_capacity = self.estimate_capacity();
        let mut tally = IndexMap::with_capacity(estimated_capacity);
        for line in reader.lines().map_while(Result::ok) {
            for word in line.unicode_words() {
                *tally.entry(case.normalize(word)).or_insert(0) += 1;
            }
        }
        tally
    }

    /// Initializes the rayon thread pool with configuration from Config
    fn init_thread_pool(threads: Threads) {
        static INIT_THREAD_POOL: std::sync::Once = std::sync::Once::new();

        match threads {
            Threads::Count(count) => {
                INIT_THREAD_POOL.call_once(|| {
                    let num_threads = count as usize;
                    if let Err(e) = rayon::ThreadPoolBuilder::new()
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

    /// Parallel implementation for larger inputs with optimized chunking strategy
    fn tally_map_parallel<T: Read>(
        &self,
        reader: BufReader<T>,
        chunk_size: u32,
        case: Case,
    ) -> IndexMap<Box<str>, usize> {
        let estimated_capacity = self.estimate_capacity();
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
