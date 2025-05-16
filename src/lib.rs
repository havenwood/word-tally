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
//! ## Module structure
//!
//! The `WordTally` library is organized into these modules:
//! - `args.rs`: CLI argument parsing and command-line interface
//! - `errors.rs`: Error types and handling
//! - `input.rs`: Input source management strategies
//! - `lib.rs`: Core library functionality and API
//! - `main.rs`: CLI entry point and execution
//! - `options/`: Configuration and processing options
//!   - `options/case.rs`: Text case normalization utilities
//!   - `options/filters.rs`: Word filtering mechanisms
//!   - `options/io.rs`: I/O operations implementation
//!   - `options/mod.rs`: Common options functionality
//!   - `options/performance.rs`: Optimization and benchmarking
//!   - `options/processing.rs`: Text processing algorithms
//!   - `options/serialization.rs`: Data serialization for exports
//!   - `options/sort.rs`: Word frequency sorting strategies
//! - `output.rs`: Output formatting and display
//! - `patterns.rs`: Regular expression and pattern matching
//! - `verbose.rs`: Logging and diagnostic information
//!
//! # Options
//!
//! The [`Options`] struct provides a unified interface for configuring all aspects of word tallying:
//!
//! ```
//! use word_tally::{Options, WordTally, Input, Io, Processing};
//! use anyhow::Result;
//!
//! # fn example() -> Result<()> {
//! // Use default options
//! let options = Options::default();
//! let file_path = std::path::Path::new("example.txt");
//! let input = Input::new(file_path, Io::Buffered)?;
//! let words = WordTally::new(&input, &options)?;
//! assert_eq!(words.count(), 9);
//! # Ok(())
//! # }
//! ```
//!
//! ## Formatting
//!
//! Controls how words are normalized, results are ordered, and output is formatted:
//!
//! * [`Case`]: Normalize word case (`Original`, `Lower`, or `Upper`)
//! * [`Sort`]: Order results by frequency (`Unsorted`, `Desc`, or `Asc`)
//! * [`Format`]: Specify output format (`Text`, `CSV`, `JSON`)
//! * [`Serialization`]: Configure output details like format and delimiters
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
//! * [`Processing`]: Choose between sequential or parallel processing
//! * [`Io`]: Control the input method (streamed, buffered, or memory-mapped)
//! * [`Threads`]: Control the thread pool size for parallel execution
//! * [`Performance`]: Configure performance settings
//!
//! ## Output
//!
//! Output the results:
//!
//! * [`Output`]: Generate formatted output based on the specified format in `Serialization`
//!
//! # Examples
//!
//! ```
//! use word_tally::{Case, Filters, Format, Options, Processing, Tally, WordTally, Input, Io};
//! use anyhow::Result;
//!
//! # fn example() -> Result<()> {
//! // Create options with case normalization, output format, and other settings
//! let options = Options::default()
//!     .with_case(Case::Lower)
//!     .with_format(Format::Json)
//!     .with_filters(Filters::default().with_min_chars(3))
//!     .with_processing(Processing::Parallel);
//!
//! let file_path = std::path::Path::new("example_word.txt");
//! let input = Input::new(file_path, Io::Buffered)?;
//! let words = WordTally::new(&input, &options)?;
//! let expected_tally: Tally = [("cinquedea".into(), 1)].into();
//!
//! assert_eq!(words.into_tally(), expected_tally);
//! # Ok(())
//! # }
//! ```

use std::{
    hash::Hash,
    io::{BufRead, Read},
    str,
    sync::Arc,
};

use anyhow::{Context, Result};
use indexmap::IndexMap;
use memmap2::Mmap;
use rayon::prelude::*;
use serde::{self, Deserialize, Serialize};
use unicode_segmentation::UnicodeSegmentation;

pub mod errors;
pub mod input;
pub mod options;
pub mod output;
pub mod patterns;

pub use input::{Input, InputReader};
pub use options::{
    Options,
    case::Case,
    filters::{ExcludeWords, Filters, MinChars, MinCount},
    io::Io,
    performance::{CHARS_PER_LINE, Performance},
    processing::{Processing, SizeHint, Threads},
    serialization::{Format, Serialization},
    sort::Sort,
};
pub use output::Output;
pub use patterns::{ExcludePatterns, IncludePatterns};

pub type Count = usize;
pub type Word = Box<str>;
pub type Tally = Box<[(Word, Count)]>;
pub type TallyMap = IndexMap<Word, Count>;

/// A shared `OnceLock` for default `Options`.
static DEFAULT_OPTIONS: std::sync::OnceLock<Options> = std::sync::OnceLock::new();

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
#[non_exhaustive]
/// A tally of word frequencies and counts, along with processing options.
pub struct WordTally<'a> {
    /// Ordered pairs of words and the count of times they appear.
    tally: Tally,

    /// All of the options specified for how to tally.
    options: &'a Options,

    /// The sum of all words tallied.
    count: Count,

    /// The sum of uniq words tallied.
    uniq_count: Count,
}

/// A default `WordTally` is empty with default `Options`.
impl Default for WordTally<'static> {
    fn default() -> Self {
        Self {
            tally: Box::new([]),
            options: DEFAULT_OPTIONS.get_or_init(Options::default),
            count: 0,
            uniq_count: 0,
        }
    }
}

/// Serializes all fields of a `WordTally` to JSON.
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

/// Deserialize into a `WordTally` from JSON.
///
/// Warning: To avoid `Options` issues with deserialization & lifetimes, we deserialize `Options`
/// by leaking the allocation to get a static reference.
impl<'de> Deserialize<'de> for WordTally<'_> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Fields {
            tally: Tally,
            options: Options,
            count: Count,
            #[serde(rename = "uniqueCount")]
            uniq_count: Count,
        }

        // Deserialize into the helper struct
        let data = Fields::deserialize(deserializer)?;

        // Leak the `Options` to obtain a reference with static lifetime for `WordTally`
        let options_ref: &'static Options = Box::leak(Box::new(data.options));

        Ok(Self {
            tally: data.tally,
            options: options_ref,
            count: data.count,
            uniq_count: data.uniq_count,
        })
    }
}

/// A `tally` supports `iter` and can also be represented as a `Vec`.
impl<'a> From<WordTally<'a>> for Vec<(Word, Count)> {
    fn from(word_tally: WordTally<'a>) -> Self {
        word_tally.into_tally().into_vec()
    }
}

/// A `tally` can also be iterated over directly from a `WordTally`.
impl<'i> IntoIterator for &'i WordTally<'_> {
    type Item = &'i (Word, Count);
    type IntoIter = std::slice::Iter<'i, (Word, Count)>;
    fn into_iter(self) -> Self::IntoIter {
        self.tally.iter()
    }
}

/// `WordTally` fields are eagerly populated upon construction and exposed by getter methods.
impl<'a> WordTally<'a> {
    /// Constructs a `WordTally` from an input source and tallying options.
    ///
    /// This constructor handles all I/O strategies (streamed, buffered and memory-mapped).
    ///
    /// # Errors
    ///
    /// An error will be returned if:
    /// - The input contains invalid UTF-8
    /// - An I/O error occurs while reading from the source
    /// - Memory mapping fails (piped input will always fail)
    ///
    /// # Examples
    ///
    /// ```
    /// use word_tally::{Options, WordTally, Input, Io, Processing};
    /// use anyhow::Result;
    ///
    /// # fn example() -> Result<()> {
    /// let options = Options::default();
    /// let input = Input::new("document.txt", Io::Streamed)?;
    /// let word_tally = WordTally::new(&input, &options)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(input: &Input, options: &'a Options) -> Result<Self> {
        // Initialize thread pool if parallel processing
        if matches!(options.processing(), Processing::Parallel) {
            options.performance().init_thread_pool();
        }

        // Generate the tally map from the input
        let tally_map = Self::tally_map_from(input, options)?;

        Ok(Self::from_tally_map(tally_map, options))
    }

    /// Creates a `WordTally` instance from a `TallyMap` and `Options`.
    fn from_tally_map(mut tally_map: TallyMap, options: &'a Options) -> Self {
        options.filters().apply(&mut tally_map, options.case());

        let count = tally_map.values().sum();
        let tally: Box<[_]> = tally_map.into_iter().collect();
        let uniq_count = tally.len();

        let mut instance = Self {
            options,
            tally,
            count,
            uniq_count,
        };

        instance.sort(options.sort());

        instance
    }

    /// Gets the `tally` field.
    pub fn tally(&self) -> &[(Word, Count)] {
        &self.tally
    }

    /// Gets a reference to the `options`.
    pub const fn options(&self) -> &Options {
        self.options
    }

    /// Gets the `uniq_count` field.
    pub const fn uniq_count(&self) -> Count {
        self.uniq_count
    }

    /// Gets the `count` field.
    pub const fn count(&self) -> Count {
        self.count
    }

    /// Consumes the `tally` field.
    pub fn into_tally(self) -> Tally {
        self.tally
    }

    /// Sorts the `tally` field in place if a sort order other than `Unsorted` is provided.
    pub fn sort(&mut self, sort: Sort) {
        sort.apply(self);
    }

    /// Creates a `TallyMap` from an input reader and options.
    fn tally_map_from(input: &Input, options: &Options) -> Result<TallyMap> {
        match (options.processing(), options.io()) {
            // Sequential processing
            (Processing::Sequential, Io::Streamed | Io::MemoryMapped) => {
                Self::streamed_count(input, options)
            }
            (Processing::Sequential, Io::Buffered | Io::Bytes) => {
                Self::buffered_count(input, options)
            }

            // Parallel processing
            (Processing::Parallel, Io::MemoryMapped) => {
                let Input::Mmap(mmap_arc, _) = input else {
                    unreachable!("This will only be called with `Input::Mmap(Arc<Mmap>, PathBuf)`.")
                };
                Self::par_mmap_count(mmap_arc, options)
            }
            (Processing::Parallel, Io::Streamed) => Self::par_streamed_count(input, options),
            (Processing::Parallel, Io::Buffered | Io::Bytes) => {
                Self::par_buffered_count(input, options)
            }
        }
    }

    //
    // Sequential I/O
    //

    /// Sequential streamed word tallying.
    ///
    /// Streams one line at a time, avoiding needing to always hold the entire input in memory.
    fn streamed_count(input: &Input, options: &Options) -> Result<TallyMap> {
        let reader = input
            .reader()
            .context("Failed to create reader for input")?;
        let mut tally = TallyMap::with_capacity(options.performance().tally_map_capacity());

        reader.lines().try_for_each(|try_line| {
            let line = try_line.context("Error reading input stream")?;
            Self::count_words(&mut tally, &line, options.case());

            Ok::<_, anyhow::Error>(())
        })?;

        Ok(tally)
    }

    /// Sequential buffered word tallying.
    ///
    /// Reads the entire input into memory before processing sequentially.
    fn buffered_count(input: &Input, options: &Options) -> Result<TallyMap> {
        let perf = options.performance();
        let buffered_input = Self::buffer_input(input, perf)?;

        let capacity = perf.tally_map_capacity_for_buffered(buffered_input.len());
        let mut tally = TallyMap::with_capacity(capacity);
        let case = options.case();

        buffered_input.lines().for_each(|line| {
            Self::count_words(&mut tally, line, case);
        });

        Ok(tally)
    }

    //
    // Parallel I/O
    //

    /// Parallel streamed word tallying.
    ///
    /// Streams batches of lines, processing the batches in parallel. Avoids always needing to
    /// hold the entire input in memory. Uses memory-based batching for more consistent workloads.
    fn par_streamed_count(input: &Input, options: &Options) -> Result<TallyMap> {
        let perf = options.performance();
        let case = options.case();
        let reader = input
            .reader()
            .context("Failed to create reader for input")?;
        let estimated_lines_per_chunk = perf.estimated_lines_per_chunk();
        let mut tally = TallyMap::with_capacity(perf.tally_map_capacity());

        // Helper closure to process accumulated lines and merge into tally
        let mut process_batch = |batch: Vec<String>| {
            if batch.is_empty() {
                return;
            }

            // Calculate batch size in bytes for better capacity estimation
            let batch_size_bytes = batch.iter().map(|s| s.len()).sum();
            let batch_capacity = perf.tally_map_capacity_for_content(batch_size_bytes);

            // Process batch in parallel - let Rayon handle work distribution
            let batch_result = batch
                .chunks(rayon::current_num_threads() * 2)
                .par_bridge()
                .map(|chunk| {
                    let mut local_tally = TallyMap::with_capacity(batch_capacity);
                    for line in chunk {
                        WordTally::count_words(&mut local_tally, line, case);
                    }
                    local_tally
                })
                .reduce(
                    || TallyMap::with_capacity(batch_capacity),
                    WordTally::merge_tally_maps,
                );

            // Reserve space to minimize hash table resizing during merge
            tally.reserve(batch_result.len());
            Self::merge_counts(&mut tally, batch_result);
        };

        let mut batch_of_lines = Vec::with_capacity(estimated_lines_per_chunk);
        let mut accumulated_bytes = 0;

        reader.lines().try_for_each(|try_line| {
            let line = try_line.context("Error reading input stream")?;

            // Track memory used by this line
            accumulated_bytes += line.len();
            batch_of_lines.push(line);

            // Process batch when it reaches target memory threshold
            if accumulated_bytes >= perf.chunk_size() {
                // Last batch's size rather than estimation to better match input pattern
                let current_batch_size = batch_of_lines.len();
                // Swap out the full batch for an empty one, reusing the previous capacity
                let full_batch_of_lines =
                    std::mem::replace(&mut batch_of_lines, Vec::with_capacity(current_batch_size));
                process_batch(full_batch_of_lines);
                accumulated_bytes = 0;
            }

            Ok::<_, anyhow::Error>(())
        })?;

        // Process any remaining lines in the final batch
        if !batch_of_lines.is_empty() {
            process_batch(batch_of_lines);
        }

        Ok(tally)
    }

    /// Parallel memory-mapped word tallying.
    ///
    /// Uses a simple divide-and-conquer approach for parallel processing with mmap:
    /// - Divides content into chunks at UTF-8 character boundaries
    /// - Memory mapping gives direct file content access with OS-managed paging
    /// - Process full chunks without line-by-line iteration
    ///
    /// An alternative single-pass approach could begin processing chunks during
    /// boundary detection and not store chunk indices, but would need more complex
    /// thread coordination.
    fn par_mmap_count(mmap: &Arc<Mmap>, options: &Options) -> Result<TallyMap> {
        let perf = options.performance();
        let case = options.case();
        let chunk_size = perf.chunk_size();

        // This provides a view into the content rather than copying
        let content = str::from_utf8(mmap).context("Memory-mapped file contains invalid UTF-8")?;
        let total_size = content.len();
        let num_chunks = total_size.div_ceil(chunk_size);

        // Create chunk boundaries with first (0) and last (total_size) elements, plus interior boundaries
        let chunk_boundaries = {
            let mut boundaries = Vec::with_capacity(num_chunks + 1);
            // First boundary
            boundaries.push(0);

            boundaries.extend((1..num_chunks).map(|i| {
                // Find next valid UTF-8 character boundary after `chunk_size` bytes using an iterator
                (i * chunk_size..)
                    .find(|&pos| pos >= total_size || content.is_char_boundary(pos))
                    .unwrap_or(total_size)
            }));

            // Last boundary
            boundaries.push(total_size);

            boundaries
        };

        // Calculate optimal capacities for better memory usage
        let avg_chunk_size = total_size / num_chunks.max(1);
        let per_chunk_capacity = perf.tally_map_capacity_for_content(avg_chunk_size);

        // Calculate optimal capacity for the final reduced `TallyMap`
        let reduce_capacity = perf.tally_map_capacity();

        let tally = chunk_boundaries
            .windows(2)
            .par_bridge()
            .map(|window| {
                let chunk = &content[window[0]..window[1]];
                let mut local_tally = TallyMap::with_capacity(per_chunk_capacity);

                // Count words directly from chunks, without heed for newlines
                Self::count_words(&mut local_tally, chunk, case);

                local_tally
            })
            .reduce(
                || TallyMap::with_capacity(reduce_capacity),
                Self::merge_tally_maps,
            );

        Ok(tally)
    }

    /// Parallel buffered word tallying.
    ///
    /// Reads the entire input into memory before processing in parallel.
    fn par_buffered_count(input: &Input, options: &Options) -> Result<TallyMap> {
        let case = options.case();
        let perf = options.performance();
        let buffered_input = Self::buffer_input(input, perf)?;

        let tally = buffered_input
            .par_lines()
            .fold(
                || TallyMap::with_capacity(perf.per_thread_tally_map_capacity()),
                |mut tally, line| {
                    Self::count_words(&mut tally, line, case);
                    tally
                },
            )
            .reduce_with(Self::merge_tally_maps)
            .unwrap_or_else(|| TallyMap::with_capacity(perf.tally_map_capacity()));

        Ok(tally)
    }

    //
    // Helpers
    //

    /// Reads from the `Input` into a `String`.
    fn buffer_input(input: &Input, performance: &Performance) -> Result<String> {
        let capacity = performance.content_buffer_capacity();
        let mut content = String::with_capacity(capacity);

        // Create a reader from the input
        let mut reader = input
            .reader()
            .context("Failed to create reader for input")?;

        // Read from the input into the presized buffer
        reader
            .read_to_string(&mut content)
            .context("Failed to read input into buffer")?;

        Ok(content)
    }

    /// Counts words in a byte slice and adds them to the tally map.
    fn count_words(tally: &mut TallyMap, content: &str, case: Case) {
        for word in content.unicode_words() {
            let normalized = case.normalize(word);
            *tally.entry(normalized).or_insert(0) += 1;
        }
    }

    /// Merging two subtallies, combining their counts.
    fn merge_counts(dest: &mut TallyMap, source: TallyMap) {
        for (word, count) in source {
            *dest.entry(word).or_insert(0) += count;
        }
    }

    /// Merge two tally maps, always merging the smaller into the larger.
    ///
    /// Returns the merged map containing the combined counts from both input maps.
    fn merge_tally_maps(mut a: TallyMap, b: TallyMap) -> TallyMap {
        if a.len() < b.len() {
            let mut merged = b;
            merged.reserve(a.len());
            Self::merge_counts(&mut merged, a);
            merged
        } else {
            a.reserve(b.len());
            Self::merge_counts(&mut a, b);
            a
        }
    }
}
