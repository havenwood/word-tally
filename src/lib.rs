//! A tally of words with a count of the number of times each appears.
//!
//! `WordTally` tallies the occurrences of words in text sources. It operates by streaming
//! input line by line, eliminating the need to load entire files or streams into memory.
//!
//! Word boundaries are determined using the [`unicode-segmentation`](https://docs.rs/unicode-segmentation/)
//! crate, which implements the [Unicode Standard Annex #29](https://unicode.org/reports/tr29/)
//! specification for text segmentation across languages.
//!
//! The library offers both sequential and parallel processing modes. When operating
//! in parallel mode, the input is processed in discrete chunks across available CPU cores,
//! maintaining memory efficiency while improving processing speed for larger inputs.
//!
//! # Options
//!
//! The [`Options`] struct provides a unified interface for configuring all aspects of word tallying:
//!
//! ```
//! use word_tally::{Options, WordTally};
//!
//! // Use default options
//! let options = Options::default();
//! let input = "The quick brown fox jumps over the lazy dog".as_bytes();
//! let words = WordTally::new(input, &options);
//! ```
//!
//! ## Formatting
//!
//! Controls how words are normalized, results are ordered, and output is formatted:
//!
//! * [`Case`]: Normalize word case (`Original`, `Lower`, or `Upper`)
//! * [`Sort`]: Order results by frequency (`Unsorted`, `Desc`, or `Asc`)
//! * [`Format`]: Specify output format (`Text`, `CSV`, `JSON`)
//!
//! ## Filters
//!
//! Determine which words appear in the final tally:
//!
//! * Length filters: [`MinChars`] excludes words shorter than specified
//! * Frequency filters: [`MinCount`] includes only words appearing more than N times
//! * Pattern matching: [`IncludePatterns`] and [`ExcludePatterns`] for regex-based filtering
//! * Word lists: [`ExcludeWords`] for explicit exclusion of specific terms
//!
//! ## Performance
//!
//! Optimize execution for different workloads:
//!
//! * [`Concurrency`]: Choose between sequential or parallel processing
//! * [`Threads`]: Control the thread pool size for parallel execution
//! * [`SizeHint`]: Tune collection capacity pre-allocation
//!
//! ## Output
//!
//! Output the results:
//!
//! * [`Output`]: Generate formatted output based on the specified format in `Formatting`
//!
//! # Examples
//!
//! ```
//! use word_tally::{Case, Filters, Format, Options, WordTally};
//!
//! // Create options with case normalization, output format, and minimum character length filter
//! let options = Options::default()
//!     .with_case(Case::Lower)
//!     .with_format(Format::Json)
//!     .with_filters(Filters::default().with_min_chars(3));
//!
//! let input = "Cinquedea".as_bytes();
//! let words = WordTally::new(input, &options);
//! let expected_tally: Box<[(Box<str>, usize)]> = [("cinquedea".into(), 1)].into();
//!
//! assert_eq!(words.into_tally(), expected_tally);
//! ```
use anyhow::{Context, Result};
use indexmap::IndexMap;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Read};
use unicode_segmentation::UnicodeSegmentation;

pub mod filters;
pub mod formatting;
pub mod input;
pub mod options;
pub mod output;
pub mod performance;

pub use filters::{
    ExcludePatterns, ExcludeWords, Filters, IncludePatterns, MinChars, MinCount, MinValue,
};
pub use formatting::{Case, Format, Formatting, Sort};
pub use input::Input;
pub use options::Options;
pub use output::Output;
pub use performance::{Concurrency, Performance, SizeHint, Threads};

#[derive(Deserialize)]
struct WordTallyData {
    tally: Box<[(Box<str>, usize)]>,
    #[serde(default, skip_deserializing)]
    _options: (),
    count: usize,
    #[serde(rename = "uniqueCount")]
    uniq_count: usize,
}

#[derive(Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct WordTally<'a> {
    /// Ordered pairs of words and the count of times they appear.
    tally: Box<[(Box<str>, usize)]>,

    /// Options for tallying, including formatting, filters, and performance settings.
    options: &'a Options,

    /// The sum of all words tallied.
    count: usize,

    /// The sum of uniq words tallied.
    uniq_count: usize,
}

impl std::hash::Hash for WordTally<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.tally.hash(state);
        self.count.hash(state);
        self.uniq_count.hash(state);
    }
}

impl Default for WordTally<'static> {
    fn default() -> Self {
        static DEFAULT_OPTIONS: std::sync::OnceLock<Options> = std::sync::OnceLock::new();

        Self {
            tally: Box::new([]),
            options: DEFAULT_OPTIONS.get_or_init(Options::default),
            count: 0,
            uniq_count: 0,
        }
    }
}

impl Serialize for WordTally<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("WordTally", 4)?;
        state.serialize_field("tally", &self.tally)?;
        state.serialize_field("options", &self.options)?;
        state.serialize_field("count", &self.count)?;
        state.serialize_field("uniqueCount", &self.uniq_count)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for WordTally<'_> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let data = WordTallyData::deserialize(deserializer)?;
        static DEFAULT_OPTIONS: std::sync::OnceLock<Options> = std::sync::OnceLock::new();
        let options = DEFAULT_OPTIONS.get_or_init(Options::default);

        Ok(Self {
            tally: data.tally,
            options,
            count: data.count,
            uniq_count: data.uniq_count,
        })
    }
}

/// A `tally` supports `iter` and can also be represented as a `Vec`.
impl<'a> From<WordTally<'a>> for Vec<(Box<str>, usize)> {
    fn from(word_tally: WordTally<'a>) -> Self {
        word_tally.into_tally().into_vec()
    }
}

/// A `tally` can also be iterated over directly from a `WordTally`.
impl<'i> IntoIterator for &'i WordTally<'_> {
    type Item = &'i (Box<str>, usize);
    type IntoIter = std::slice::Iter<'i, (Box<str>, usize)>;
    fn into_iter(self) -> Self::IntoIter {
        self.tally.iter()
    }
}

/// `WordTally` fields are eagerly populated upon construction and exposed by getter methods.
impl<'a> WordTally<'a> {
    /// Constructs a new `WordTally` from a source that implements `Read`.
    ///
    /// Takes a reference to unified options that include formatting, filters, and performance settings.
    pub fn new<T: Read>(input: T, options: &'a Options) -> Self {
        // Initialize thread pool if using parallel processing
        if matches!(options.concurrency(), Concurrency::Parallel) {
            options.performance().init_thread_pool();
        }

        let reader = BufReader::new(input);
        let mut instance = Self {
            options,
            tally: Box::new([]),
            count: 0,
            uniq_count: 0,
        };

        let mut tally_map = match options.concurrency() {
            Concurrency::Sequential => instance.tally_map(reader, options.case()),
            Concurrency::Parallel => instance.tally_map_parallel(
                reader,
                options.performance().chunk_size(),
                options.case(),
            ),
        };

        options.filters().apply(&mut tally_map, options.case());

        let count = tally_map.values().sum();
        let tally: Box<[_]> = tally_map.into_iter().collect();
        let uniq_count = tally.len();

        instance.tally = tally;
        instance.count = count;
        instance.uniq_count = uniq_count;
        instance.sort(options.sort());

        instance
    }

    /// Deserializes a WordTally from a JSON string.
    ///
    /// # Errors
    ///
    /// Returns an error if the JSON string contains invalid syntax or missing required fields.
    pub fn from_json_str(json_str: &str, options: &'a Options) -> Result<Self> {
        let data: WordTallyData = serde_json::from_str(json_str)
            .context("Failed to deserialize WordTally from JSON string")?;

        Ok(Self {
            tally: data.tally,
            options,
            count: data.count,
            uniq_count: data.uniq_count,
        })
    }

    /// Deserializes a WordTally from a JSON reader.
    ///
    /// Returns an error if the JSON contains invalid syntax, missing required fields,
    /// or if an I/O error occurs while reading.
    pub fn from_json_reader<R: std::io::Read>(reader: R, options: &'a Options) -> Result<Self> {
        let data: WordTallyData = serde_json::from_reader(reader)
            .context("Failed to deserialize WordTally from reader")?;

        Ok(Self {
            tally: data.tally,
            options,
            count: data.count,
            uniq_count: data.uniq_count,
        })
    }

    /// Consumes the `tally` field.
    pub fn into_tally(self) -> Box<[(Box<str>, usize)]> {
        self.tally
    }

    /// Gets the `tally` field.
    pub const fn tally(&self) -> &[(Box<str>, usize)] {
        &self.tally
    }

    /// Gets a reference to the options.
    pub const fn options(&self) -> &Options {
        self.options
    }

    /// Gets the filters from the options.
    pub const fn filters(&self) -> &Filters {
        self.options.filters()
    }

    /// Gets the `uniq_count` field.
    pub const fn uniq_count(&self) -> usize {
        self.uniq_count
    }

    /// Gets the `count` field.
    pub const fn count(&self) -> usize {
        self.count
    }

    /// Sorts the `tally` field in place if a sort order other than `Unsorted` is provided.
    pub fn sort(&mut self, sort: Sort) {
        sort.apply(self);
    }

    /// Sequential implementation for word tallying
    fn tally_map<T: Read>(&self, reader: BufReader<T>, case: Case) -> IndexMap<Box<str>, usize> {
        let estimated_capacity = self.options().performance().estimate_capacity();
        let mut tally = IndexMap::with_capacity(estimated_capacity);
        for line in reader.lines().map_while(Result::ok) {
            for word in line.unicode_words() {
                *tally.entry(case.normalize(word)).or_insert(0) += 1;
            }
        }
        tally
    }

    /// Parallel implementation for larger inputs with optimized chunking strategy
    fn tally_map_parallel<T: Read>(
        &self,
        reader: BufReader<T>,
        chunk_size: u32,
        case: Case,
    ) -> IndexMap<Box<str>, usize> {
        let performance = self.options().performance();
        let estimated_capacity = performance.estimate_capacity();
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

    /// Processes a batch of lines in parallel
    fn process_batch(
        &self,
        lines: &[String],
        case: Case,
        estimated_capacity: usize,
        num_threads: usize,
    ) -> IndexMap<Box<str>, usize> {
        let performance = self.options().performance();
        let chunk_size = std::cmp::max(4, lines.len() / num_threads.max(1));
        let thread_maps: Vec<IndexMap<Box<str>, usize>> = lines
            .par_chunks(chunk_size)
            .map(|chunk| {
                let mut local_counts =
                    IndexMap::with_capacity(performance.estimate_chunk_capacity(chunk.len()));
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
}
