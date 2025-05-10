//! A tally of words with a count of the number of times each appears.
//!
//! `WordTally` tallies the occurrences of words in text sources. It operates by streaming
//! input line by line, eliminating the need to load entire files or streams into memory.
//!
//! Word boundaries are determined using the [`bstr`](https://docs.rs/bstr/)
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
//! use anyhow::Result;
//! use std::io::Cursor;
//!
//! # fn example() -> Result<()> {
//! // Use default options
//! let options = Options::default();
//! let input = Cursor::new("The quick brown fox jumps over the lazy dog");
//! let words = WordTally::new(input, &options)?;
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
//! use word_tally::{Case, Filters, Format, Io, Options, Processing, Tally, WordTally};
//! use anyhow::Result;
//! use std::io::Cursor;
//!
//! # fn example() -> Result<()> {
//! // Create options with case normalization, output format, and other settings
//! let options = Options::default()
//!     .with_case(Case::Lower)
//!     .with_format(Format::Json)
//!     .with_filters(Filters::default().with_min_chars(3))
//!     .with_processing(Processing::Parallel);
//!
//! let input = Cursor::new("Cinquedea");
//! let words = WordTally::new(input, &options)?;
//! let expected_tally: Tally = [("cinquedea".into(), 1)].into();
//!
//! assert_eq!(words.into_tally(), expected_tally);
//! # Ok(())
//! # }
//! ```
use anyhow::{Context, Result};
use bstr::{ByteSlice, io::BufReadExt};
use indexmap::IndexMap;
use memmap2::Mmap;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, Read};

pub type Word = Box<str>;
pub type Count = usize;
pub type Tally = Box<[(Word, Count)]>;
type TallyMap = IndexMap<Word, Count>;

pub mod errors;
pub mod filters;
pub mod formatting;
pub mod input;
pub mod options;
pub mod output;
pub mod performance;

pub use filters::{ExcludePatterns, ExcludeWords, Filters, IncludePatterns, MinChars, MinCount};
pub use formatting::{Case, Format, Formatting, Sort};
pub use input::Input;
pub use options::Options;
pub use output::Output;
pub use performance::{Io, Performance, Processing, SizeHint, Threads};

// Shared OnceLock for default Options
static DEFAULT_OPTIONS: std::sync::OnceLock<Options> = std::sync::OnceLock::new();

#[derive(Deserialize)]
struct WordTallyData {
    tally: Tally,
    count: Count,
    #[serde(rename = "uniqueCount")]
    uniq_count: Count,
}

#[derive(Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct WordTally<'a> {
    /// Ordered pairs of words and the count of times they appear.
    tally: Tally,

    /// Options for tallying, including formatting, filters, and performance settings.
    options: &'a Options,

    /// The sum of all words tallied.
    count: Count,

    /// The sum of uniq words tallied.
    uniq_count: Count,
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
    /// Constructs a `WordTally` from any input source that implements `Read`.
    ///
    /// This constructor handles `streamed` and `buffered` I/O strategies according
    /// to the provided options and properly propagates any errors that may occur.
    ///
    /// For `memory-mapped` I/O, use the `from_file()` method which takes a `File` reference.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - Memory-mapped I/O is requested (use `from_file()` instead)
    /// - The input contains invalid UTF-8
    /// - An I/O error occurs while reading from the source
    ///
    /// # Examples
    ///
    /// ```
    /// use word_tally::{Options, WordTally};
    /// use std::io::Cursor;
    /// use anyhow::Result;
    ///
    /// # fn example() -> Result<()> {
    /// // Basic usage with any Read input
    /// let options = Options::default();
    /// let input = Cursor::new("The quick brown fox");
    /// let word_tally = WordTally::new(input, &options)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new<T: Read>(input: T, options: &'a Options) -> Result<Self> {
        Self::init_thread_pool_if_parallel(options);

        let tally_map = match (options.io(), options.processing()) {
            // Memory-mapped I/O requires a `File` rather than implementing `Read`
            (Io::MemoryMapped, _) => {
                return Err(anyhow::anyhow!(
                    "Memory-mapped I/O requires a file input. Use a file path instead of stdin."
                ));
            }

            // Streamed I/O
            (Io::Streamed, Processing::Sequential) => Self::count_streamed(input, options)?,
            (Io::Streamed, Processing::Parallel) => Self::count_streamed_parallel(input, options)?,

            // Buffered I/O
            (Io::Buffered, Processing::Sequential) => Self::count_buffered(input, options)?,
            (Io::Buffered, Processing::Parallel) => Self::count_buffered_parallel(input, options)?,
        };

        Ok(Self::from_tally_map(tally_map, options))
    }

    /// Creates a `WordTally` from a `File` reference.
    ///
    /// Works with all I/O strategies (memory-mapped, streamed, and buffered) using
    /// a `File` input. For non-`File` inputs, use the `new()` method instead.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The file contains invalid UTF-8
    /// - An I/O error occurs while reading from the file
    /// - Memory mapping fails (when using `Io::MemoryMapped`)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use word_tally::{Options, WordTally, Io, Processing};
    /// use std::fs::File;
    /// use anyhow::Result;
    ///
    /// # fn example() -> Result<()> {
    /// // Using memory-mapped I/O with a File
    /// let options = Options::default()
    ///     .with_io(Io::MemoryMapped)
    ///     .with_processing(Processing::Parallel);
    /// let file = File::open("large-file.txt")?;
    /// let word_tally = WordTally::from_file(&file, &options)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_file(file: &File, options: &'a Options) -> Result<Self> {
        Self::init_thread_pool_if_parallel(options);

        let tally_map = match (options.io(), options.processing()) {
            // Memory-mapped I/O requires a `File`
            (Io::MemoryMapped, Processing::Sequential) => Self::count_mmap(file, options)?,
            (Io::MemoryMapped, Processing::Parallel) => Self::count_mmap_parallel(file, options)?,

            // Streamed I/O also works with `new()`
            (Io::Streamed, Processing::Sequential) => Self::count_streamed(file, options)?,
            (Io::Streamed, Processing::Parallel) => Self::count_streamed_parallel(file, options)?,

            // Buffered I/O also works with `new()`
            (Io::Buffered, Processing::Sequential) => Self::count_buffered(file, options)?,
            (Io::Buffered, Processing::Parallel) => Self::count_buffered_parallel(file, options)?,
        };

        Ok(Self::from_tally_map(tally_map, options))
    }

    /// Create a thread pool if `Processing::Parallel`.
    fn init_thread_pool_if_parallel(options: &Options) {
        if matches!(options.processing(), Processing::Parallel) {
            options.performance().init_thread_pool();
        }
    }

    /// Creates a `WordTally` instance from `TallyMap` and `Options`.
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

    /// Create `WordTally` from deserialized data
    fn from_deserialized_data(data: WordTallyData, options: &'a Options) -> Self {
        Self {
            tally: data.tally,
            options,
            count: data.count,
            uniq_count: data.uniq_count,
        }
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

    /// Gets the `tally` field.
    pub fn tally(&self) -> &[(Word, Count)] {
        &self.tally
    }

    /// Gets a reference to the `options`.
    pub const fn options(&self) -> &Options {
        self.options
    }

    /// Gets the `filters` from the `options`.
    pub const fn filters(&self) -> &Filters {
        self.options.filters()
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

    //
    // Sequential I/O strategies
    //

    /// Sequential implementation for streamed word tallying using bstr's BufReadExt
    fn count_streamed<T: Read>(input: T, options: &Options) -> Result<TallyMap> {
        let mut reader = BufReader::new(input);
        let mut tally = TallyMap::with_capacity(options.performance().estimate_capacity());
        let case = options.case();

        // Process lines one at a time without allocating new strings per line
        reader
            .for_byte_line(|line| {
                Self::count_words(&mut tally, line, case);

                Ok(true)
            })
            .context("Error reading input stream")?;

        Ok(tally)
    }

    /// Sequential implementation for reading the input entirely into memory
    fn count_buffered<T: Read>(input: T, options: &Options) -> Result<TallyMap> {
        let content = Self::buffer_input(input)?;

        Ok(Self::process_sequential_content(&content, options))
    }

    /// Memory-mapping uses OS virtual memory for efficient file access
    fn count_mmap(file: &File, options: &Options) -> Result<TallyMap> {
        // Note: All memory-mapping require `unsafe` as per `memmap2` crate design
        let mmap = unsafe { Mmap::map(file).context("Failed to memory map the file")? };
        let content =
            std::str::from_utf8(&mmap).context("Memory-mapped file contains invalid UTF-8")?;

        Ok(Self::process_sequential_content(content, options))
    }

    //
    // Parallel I/O strategies
    //

    /// Parallel implementation for streamed processing
    fn count_streamed_parallel<T: Read>(input: T, options: &Options) -> Result<TallyMap> {
        let mut reader = BufReader::new(input);
        let perf = options.performance();
        let case = options.case();

        // Calculate batch size based on performance settings
        let batch_size = perf.chunk_size();

        // Initial capacity based on performance settings
        let mut result_map = TallyMap::with_capacity(perf.estimate_capacity());

        // Estimate more accurate line capacity based on average line length
        // This helps reduce reallocations while keeping memory usage reasonable
        let estimated_line_length = 80;
        let estimated_lines = (batch_size / estimated_line_length).max(100);
        let mut lines = Vec::with_capacity(estimated_lines);
        let mut batch_bytes = 0;

        // Stream the file line by line, processing in batches
        reader
            .for_byte_line(|line| {
                if !line.is_empty() {
                    // Add line to current batch
                    lines.push(line.to_vec());
                    batch_bytes += line.len();

                    // When batch is full, process it in parallel
                    if batch_bytes >= batch_size {
                        // Process batch using chunked parallel reduction
                        let batch_result = Self::process_line_batch(&lines, case, options);

                        // Merge batch results
                        result_map.reserve(batch_result.len());
                        Self::merge_counts(&mut result_map, batch_result);

                        // Reset for next batch
                        lines.clear();
                        batch_bytes = 0;
                    }
                }

                Ok(true)
            })
            .context("Error reading input stream")?;

        // Process any remaining lines in the final batch
        if !lines.is_empty() {
            let batch_result = Self::process_line_batch(&lines, case, options);
            result_map.reserve(batch_result.len());
            Self::merge_counts(&mut result_map, batch_result);
        }

        Ok(result_map)
    }

    /// Parallel implementation for buffered word tallying
    fn count_buffered_parallel<T: Read>(input: T, options: &Options) -> Result<TallyMap> {
        let content = Self::buffer_input(input)?;
        let perf = options.performance();
        let case = options.case();

        // Use `par_bridge` for better memory efficiency without collecting to `Vec`
        let tally = content
            .lines()
            .par_bridge()
            .fold(
                || TallyMap::with_capacity(perf.thread_count()),
                |mut map, line| {
                    Self::count_words(&mut map, line.as_bytes(), case);
                    map
                },
            )
            .reduce(
                || TallyMap::with_capacity(perf.estimate_capacity()),
                Self::merge_tally_maps,
            );

        Ok(tally)
    }

    /// Parallel implementation for memory-mapped file processing
    fn count_mmap_parallel(file: &File, options: &Options) -> Result<TallyMap> {
        // Memory-mapping uses OS virtual memory for efficient file access
        // Note: All memory-mapping require `unsafe` as per `memmap2` crate design
        let mmap = unsafe { Mmap::map(file).context("Failed to memory map the file")? };
        let content =
            std::str::from_utf8(&mmap).context("Memory-mapped file contains invalid UTF-8")?;
        let perf = options.performance();
        let case = options.case();

        // Use `par_bridge` for better memory efficiency without collecting to `Vec`
        let tally = content
            .lines()
            .par_bridge()
            .fold(
                || TallyMap::with_capacity(perf.thread_count()),
                |mut map, line| {
                    Self::count_words(&mut map, line.as_bytes(), case);
                    map
                },
            )
            .reduce(
                || TallyMap::with_capacity(perf.estimate_capacity()),
                Self::merge_tally_maps,
            );

        Ok(tally)
    }

    //
    // Count processing helpers
    //

    /// Process batch of lines in parallel
    fn process_line_batch(lines: &[Vec<u8>], case: Case, options: &Options) -> TallyMap {
        Self::parallel_reduce_chunked(
            lines,
            |map, chunk| {
                for line in chunk {
                    Self::count_words(map, line, case);
                }
            },
            options,
        )
    }

    /// Reads from input into a String, propagating any errors
    fn buffer_input<T: Read>(mut input: T) -> Result<String> {
        let mut content = String::new();
        input
            .read_to_string(&mut content)
            .context("Failed to read input into buffer")?;

        Ok(content)
    }

    /// Counts words in a byte slice and adds them to the tally map
    fn count_words(tally: &mut TallyMap, bytes: &[u8], case: Case) {
        bytes.words().for_each(|word| {
            let normalized = case.normalize(word);
            *tally.entry(normalized).or_insert(0) += 1;
        });
    }

    /// Helper function to process content sequentially line by line
    fn process_sequential_content(content: &str, options: &Options) -> TallyMap {
        let mut tally = TallyMap::with_capacity(options.performance().estimate_capacity());
        let case = options.case();

        // Process lines one at a time without allocating new strings per line
        content
            .as_bytes()
            .for_byte_line(|line| {
                Self::count_words(&mut tally, line, case);
                Ok(true)
            })
            .expect("Error processing line");

        tally
    }

    /// Merges maps by combining word counts
    fn merge_counts(dest: &mut TallyMap, source: TallyMap) {
        for (word, count) in source {
            *dest.entry(word).or_insert(0) += count;
        }
    }

    /// Merge maps efficiently by merging smaller into larger
    fn merge_tally_maps(mut left: TallyMap, mut right: TallyMap) -> TallyMap {
        if left.len() < right.len() {
            std::mem::swap(&mut left, &mut right);
        }

        left.reserve(right.len());
        Self::merge_counts(&mut left, right);
        left
    }

    /// Process data in parallel chunks with balanced workload distribution
    fn parallel_reduce_chunked<I, F>(items: &[I], chunk_fn: F, options: &Options) -> TallyMap
    where
        I: Sync,
        F: Fn(&mut TallyMap, &[I]) + Sync + Send,
    {
        let perf = options.performance();
        let num_threads = rayon::current_num_threads();
        let chunk_size = (items.len() / num_threads).max(1);

        items
            .par_chunks(chunk_size)
            .fold(
                || TallyMap::with_capacity(perf.thread_count()),
                |mut local_map, chunk| {
                    chunk_fn(&mut local_map, chunk);
                    local_map
                },
            )
            .reduce(
                || TallyMap::with_capacity(perf.estimate_capacity()),
                Self::merge_tally_maps,
            )
    }
}