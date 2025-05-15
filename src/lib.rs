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
//! - `case.rs`: Text case normalization utilities
//! - `errors.rs`: Error types and handling
//! - `filters.rs`: Word filtering mechanisms
//! - `input.rs`: Input source management strategies
//! - `io.rs`: I/O operations implementation
//! - `lib.rs`: Core library functionality and API
//! - `main.rs`: CLI entry point and execution
//! - `options.rs`: Configuration and processing options
//! - `output.rs`: Output formatting and display
//! - `patterns.rs`: Regular expression and pattern matching
//! - `performance.rs`: Optimization and benchmarking
//! - `processing.rs`: Text processing algorithms
//! - `serialization.rs`: Data serialization for exports
//! - `sort.rs`: Word frequency sorting strategies
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

use std::hash::{Hash, Hasher};
use std::io::{BufRead, Read};
use std::mem;
use std::str;
use std::sync::Arc;

use anyhow::{Context, Result};
use indexmap::IndexMap;
use memmap2::Mmap;
use rayon::prelude::*;
use serde::{self, Deserialize, Serialize};
use unicode_segmentation::UnicodeSegmentation;

pub type Count = usize;
pub type Word = Box<str>;
pub type Tally = Box<[(Word, Count)]>;
pub type TallyMap = IndexMap<Word, Count>;

pub mod case;
pub mod errors;
pub mod filters;
pub mod input;
pub mod io;
pub mod options;
pub mod output;
pub mod patterns;
pub mod performance;
pub mod processing;
pub mod serialization;
pub mod sort;

pub use case::Case;
pub use filters::{ExcludeWords, Filters, MinChars, MinCount};
pub use input::{Input, InputReader};
pub use io::Io;
pub use options::Options;
pub use output::Output;
pub use patterns::{ExcludePatterns, IncludePatterns};
pub use performance::Performance;
pub use processing::{Processing, SizeHint, Threads};
pub use serialization::{Format, Serialization};
pub use sort::Sort;

/// A shared `OnceLock` for default `Options`.
static DEFAULT_OPTIONS: std::sync::OnceLock<Options> = std::sync::OnceLock::new();

#[derive(Debug, PartialEq, Eq)]
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

/// The hash of a `WordTally` does not include its `Options`.
impl Hash for WordTally<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.tally.hash(state);
        self.count.hash(state);
        self.uniq_count.hash(state);
    }
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

/// Intermediate deserialization structure to handle `WordTally` lifetime constraint.
///
/// This is used by `from_deserialized_data()` to to deserialize data for `from_json_str()`
/// and `from_json_reader()`.
#[derive(Deserialize)]
struct WordTallyData {
    tally: Tally,
    count: Count,
    #[serde(rename = "uniqueCount")]
    uniq_count: Count,
}

/// Deserialize into a `WordTally` from JSON.
///
/// Warning: Does not properly deserialize `Options`. Instead, it uses a default `Options`.
impl<'de> Deserialize<'de> for WordTally<'_> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let data = WordTallyData::deserialize(deserializer)?;
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

    /// Deserializes a `WordTally` from a JSON string.
    ///
    /// Returns an error if the JSON string contains invalid syntax or missing required fields.
    pub fn from_json_str(json_str: &str, options: &'a Options) -> Result<Self> {
        let data: WordTallyData = serde_json::from_str(json_str)
            .context("Failed to deserialize WordTally from JSON string")?;

        Ok(Self::from_deserialized_data(data, options))
    }

    /// Deserializes a `WordTally` from a JSON reader.
    ///
    /// Returns an error if the JSON contains invalid syntax, missing required fields,
    /// or if an I/O error occurs while reading.
    pub fn from_json_reader<R: Read>(reader: R, options: &'a Options) -> Result<Self> {
        let data: WordTallyData = serde_json::from_reader(reader)
            .context("Failed to deserialize WordTally from reader")?;

        Ok(Self::from_deserialized_data(data, options))
    }

    /// A helper to create a `WordTally` from deserialized data.
    fn from_deserialized_data(data: WordTallyData, options: &'a Options) -> Self {
        Self {
            tally: data.tally,
            options,
            count: data.count,
            uniq_count: data.uniq_count,
        }
    }

    /// Constructs a `WordTally` directly from any `Read` implementation.
    ///
    /// This constructor is useful for testing or working with in-memory data.
    /// It uses the I/O strategy specified in the options.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The input contains invalid UTF-8
    /// - An I/O error occurs while reading from the source
    /// - Memory-mapped I/O is requested (this method only supports streamed or buffered I/O)
    ///
    /// # Examples
    ///
    /// ```
    /// use word_tally::{Options, WordTally};
    /// use anyhow::Result;
    /// use std::io::Cursor;
    ///
    /// # fn example() -> Result<()> {
    /// let options = Options::default();
    /// let input = Cursor::new("The quick brown fox");
    /// let words = WordTally::from_reader(input, &options)?;
    /// assert_eq!(words.count(), 4);
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_reader<R: Read + Send + Sync>(mut reader: R, options: &'a Options) -> Result<Self> {
        // Initialize thread pool if parallel processing
        if matches!(options.processing(), Processing::Parallel) {
            options.performance().init_thread_pool();
        }

        // Memory-mapped I/O doesn't work with generic readers
        if matches!(options.io(), Io::MemoryMapped) {
            anyhow::bail!(
                "Memory-mapped I/O is not supported with from_reader(). Use streamed or buffered I/O instead."
            );
        }

        // Create a bytes input from the reader content
        let mut bytes = Vec::new();
        reader
            .read_to_end(&mut bytes)
            .context("Failed to read from provided reader")?;
        let input = Input::from_bytes(bytes);

        let tally_map = Self::tally_map_from(&input, options)?;
        Ok(Self::from_tally_map(tally_map, options))
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
        let reader = input::InputReader::new(input).context("Failed to create reader for input")?;
        let mut tally = TallyMap::with_capacity(options.performance().default_tally_map_capacity());

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
        let buffered_input = Self::buffer_input(input, options.performance())?;

        let capacity = options
            .performance()
            .tally_map_capacity_for_content(buffered_input.len());
        let mut tally = TallyMap::with_capacity(capacity);
        let case = options.case();

        for line in buffered_input.lines() {
            Self::count_words(&mut tally, line, case);
        }

        Ok(tally)
    }

    //
    // Parallel I/O
    //

    /// Parallel streamed word tallying.
    ///
    /// Streams batches of lines, processing the batches in parallel. Avoids always needing to
    /// hold the entire input in memory.
    fn par_streamed_count(input: &Input, options: &Options) -> Result<TallyMap> {
        let perf = options.performance();
        let case = options.case();
        let reader = input::InputReader::new(input).context("Failed to create reader for input")?;
        let mut tally = TallyMap::with_capacity(perf.default_tally_map_capacity());
        let lines_batch_capacity = perf.lines_batch_capacity();
        let per_thread_tally_map_capacity = perf.per_thread_tally_map_capacity();

        // Prepare batch container with appropriate capacity
        let mut reusable_lines_batch = Vec::with_capacity(lines_batch_capacity);

        // Use lines iterator for cleaner syntax while keeping mem::take optimization
        reader.lines().try_fold(
            String::with_capacity(perf.line_buffer_capacity()),
            |mut reusable_line_buffer, try_line| {
                let line = try_line.context("Error reading input stream")?;

                if !line.is_empty() {
                    // Let the next line reuse the line buffer
                    reusable_line_buffer.clear();
                    // Move the line into the line buffer
                    reusable_line_buffer.push_str(&line);
                    // Add the buffer contents to the lines batch without reallocating
                    reusable_lines_batch.push(mem::take(&mut reusable_line_buffer));

                    // Process batch when it reaches optimal size
                    if reusable_lines_batch.len() >= lines_batch_capacity {
                        let batch_result = process_lines_in_parallel(
                            &reusable_lines_batch,
                            case,
                            per_thread_tally_map_capacity,
                            lines_batch_capacity,
                        );

                        // Reserve space to minimize hash table resizing during merge
                        tally.reserve(batch_result.len());
                        Self::merge_counts(&mut tally, batch_result);

                        // Let the next thread reuse the lines batch
                        reusable_lines_batch.clear();
                    }
                }

                Ok::<_, anyhow::Error>(reusable_line_buffer)
            },
        )?;

        // Process any remaining lines in the final batch
        if !reusable_lines_batch.is_empty() {
            let batch_result = process_lines_in_parallel(
                &reusable_lines_batch,
                case,
                per_thread_tally_map_capacity,
                lines_batch_capacity,
            );

            // Reserve space to minimize hash table resizing during merge
            tally.reserve(batch_result.len());
            Self::merge_counts(&mut tally, batch_result);
        }

        // Parallel helper function to process a batch of lines
        fn process_lines_in_parallel(
            reusable_lines_batch: &[String],
            case: Case,
            capacity: usize,
            lines_batch_capacity: usize,
        ) -> TallyMap {
            use rayon::prelude::*;

            reusable_lines_batch
                .par_iter()
                .with_min_len(lines_batch_capacity)
                .fold(
                    || TallyMap::with_capacity(capacity),
                    |mut tally, line| {
                        WordTally::count_words(&mut tally, line, case);
                        tally
                    },
                )
                .reduce(
                    || TallyMap::with_capacity(capacity),
                    |mut a, b| {
                        // Optimize merging by always merging the smaller map into the larger one
                        if a.len() < b.len() {
                            let mut merged = b;
                            merged.reserve(a.len());
                            WordTally::merge_counts(&mut merged, a);
                            merged
                        } else {
                            a.reserve(b.len());
                            WordTally::merge_counts(&mut a, b);
                            a
                        }
                    },
                )
        }

        Ok(tally)
    }

    /// Parallel memory-mapped word tallying.
    ///
    /// Uses a two-pass approach for parallel processing with mmap:
    /// - First pass: Scans sequentially to find newline and chunk boundary positions
    /// - Second pass: Processes each chunk in parallel, slicing on boundaries
    ///
    /// This approach has some benefits:
    /// - Memory mapping gives direct file content access with OS-managed paging
    /// - The first pass "warms" the page cache for the second
    /// - Predetermined boundaries simplify the parallel processing implementation
    ///
    /// An alternative single-pass approach could begin processing chunks during
    /// boundary detection and not store chunk indices, but would need more complex
    /// thread coordination.
    fn par_mmap_count(mmap: &Arc<Mmap>, options: &Options) -> Result<TallyMap> {
        let perf = options.performance();
        let case = options.case();

        // This provides a view into the content rather than copying
        let content = str::from_utf8(mmap).context("Memory-mapped file contains invalid UTF-8")?;

        let chunk_size = perf.chunk_size();
        let total_size = content.len();
        let num_chunks = total_size.div_ceil(chunk_size);

        // Calculate chunk boundaries that align with newlines to avoid splitting words
        let mut chunk_boundaries = Vec::with_capacity(num_chunks + 1);
        chunk_boundaries.push(0);

        let mut pos = chunk_size.min(total_size);
        while pos < total_size {
            // Ensure we're at a valid UTF-8 character boundary
            while pos < total_size && !content.is_char_boundary(pos) {
                pos += 1;
            }

            // Find the next newline after current position to ensure chunks end at line breaks
            if pos < total_size {
                // Finding `\n` and not `\r\n` is okay since `\r` wont be a grapheme
                if let Some(nl_pos) = content[pos..].find('\n') {
                    chunk_boundaries.push(pos + nl_pos + 1);
                } else {
                    // EOF without a trailing newline
                    break;
                }
            } else {
                // EOF with a trailing newline
                break;
            }

            pos += chunk_size;
            pos = pos.min(total_size);
        }

        chunk_boundaries.push(total_size);

        // Process chunks in parallel and merge the results
        // Each chunk is a range of bytes from `start` to `end` boundary that contains complete lines
        let tally = (0..chunk_boundaries.len() - 1)
            .into_par_iter()
            .map(|i| {
                let start = chunk_boundaries[i];
                let end = chunk_boundaries[i + 1];
                let chunk = &content[start..end];

                let mut local_tally = TallyMap::with_capacity(perf.per_thread_tally_map_capacity());

                // Process each line in this chunk with `unicode_words()` via `count_words()`
                for line in chunk.lines() {
                    Self::count_words(&mut local_tally, line, case);
                }

                local_tally
            })
            .reduce(
                || TallyMap::with_capacity(perf.default_tally_map_capacity()),
                |mut a, b| {
                    // Merge the smaller map into the larger one
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
                },
            );

        Ok(tally)
    }

    /// Parallel buffered word tallying.
    ///
    /// Reads the entire input into memory before processing in parallel.
    fn par_buffered_count(input: &Input, options: &Options) -> Result<TallyMap> {
        let perf = options.performance();
        let buffered_input = Self::buffer_input(input, perf)?;

        let tally = buffered_input
            .par_lines()
            .fold(
                || TallyMap::with_capacity(perf.per_thread_tally_map_capacity()),
                |mut tally, line| {
                    Self::count_words(&mut tally, line, options.case());
                    tally
                },
            )
            .reduce(
                || TallyMap::with_capacity(perf.default_tally_map_capacity()),
                |mut a, b| {
                    // Merge the smaller map into the larger
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
                },
            );

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
        let mut reader = InputReader::new(input).context("Failed to create reader for input")?;

        // Read from the input into the presized buffer
        reader
            .read_to_string(&mut content)
            .context("Failed to read input into buffer")?;

        Ok(content)
    }

    /// Counts words in a byte slice and adds them to the tally map.
    fn count_words(tally: &mut TallyMap, line: &str, case: Case) {
        for word in line.unicode_words() {
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
}
