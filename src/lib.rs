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
//!
//! // Create options with case normalization, output format, and other settings
//! let options = Options::default()
//!     .with_case(Case::Lower)
//!     .with_format(Format::Json)
//!     .with_filters(Filters::default().with_min_chars(3))
//!     .with_processing(Processing::Parallel)
//!     .with_io(Io::MemoryMapped);
//!
//! let input = "Cinquedea".as_bytes();
//! let words = WordTally::new(input, &options);
//! let expected_tally: Tally = [("cinquedea".into(), 1)].into();
//!
//! assert_eq!(words.into_tally(), expected_tally);
//! ```
use anyhow::{Context, Result};
use indexmap::IndexMap;
use memmap2::Mmap;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use unicode_segmentation::UnicodeSegmentation;

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

#[derive(Deserialize)]
struct WordTallyData {
    tally: Tally,
    #[serde(default, skip_deserializing)]
    _options: (),
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
    /// Constructs a new `WordTally` from a source that implements `Read`.
    ///
    /// Uses the I/O and processing strategies specified in the options.
    pub fn new<T: Read>(input: T, options: &'a Options) -> Self {
        // Initialize thread pool if parallel processing is selected
        if matches!(options.processing(), Processing::Parallel) {
            options.performance().init_thread_pool();
        }

        // Process input and construct a `WordTally` instance
        let tally_map = Self::process_input(input, options);
        Self::from_tally_map(tally_map, options)
    }

    /// Creates an initial tally map and processes input.
    fn process_input<T: Read>(input: T, options: &Options) -> TallyMap {
        // Show memory mapping note if that I/O mode was requested with non-file input
        if matches!(options.io(), Io::MemoryMapped) && options.performance().verbose() {
            eprintln!(
                "Note: Memory-mapped I/O requires a file source. Use WordTally::try_from_file() for memory mapping."
            );
        }

        match (options.io(), options.processing()) {
            // Streamed I/O
            (Io::Streamed, Processing::Sequential) => {
                let reader = BufReader::new(input);
                Self::tally_map_sequential_streamed(reader, options)
            }
            (Io::Streamed, Processing::Parallel) => {
                let reader = BufReader::new(input);
                Self::tally_map_parallel_streamed(reader, options)
            }

            // Buffered I/O
            (Io::Buffered, Processing::Sequential) => {
                Self::tally_map_sequential_buffered(input, options)
            }
            (Io::Buffered, Processing::Parallel) => {
                Self::tally_map_parallel_buffered(input, options)
            }

            // Memory-mapped fallback to streaming
            (Io::MemoryMapped, _) => {
                let reader = BufReader::new(input);
                Self::tally_map_sequential_streamed(reader, options)
            }
        }
    }

    /// Creates a WordTally instance from a pre-populated tally map.
    fn from_tally_map(mut tally_map: TallyMap, options: &'a Options) -> Self {
        // Apply filters
        options.filters().apply(&mut tally_map, options.case());

        // Convert results to final form
        let count = tally_map.values().sum();
        let tally: Box<[_]> = tally_map.into_iter().collect();
        let uniq_count = tally.len();

        // Create instance
        let mut instance = Self {
            options,
            tally,
            count,
            uniq_count,
        };

        // Apply sorting
        instance.sort(options.sort());

        instance
    }

    /// Constructs a new `WordTally` specifically using memory-mapped I/O.
    ///
    /// This method is specialized for memory-mapped I/O, which can provide better
    /// performance for large files by using the operating system's virtual memory system.
    ///
    /// # Errors
    ///
    /// Returns an error if memory mapping fails or if the file contains invalid UTF-8.
    pub fn try_from_file(file: File, options: &'a Options) -> Result<Self> {
        if matches!(options.processing(), Processing::Parallel) {
            options.performance().init_thread_pool();
        }

        // Process with memory-mapped I/O and construct a `WordTally` instance
        let tally_map = Self::tally_map_memory_mapped(&file, options)?;
        Ok(Self::from_tally_map(tally_map, options))
    }

    /// Deserializes a `WordTally` from a JSON string.
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

    /// Deserializes a `WordTally` from a JSON reader.
    ///
    /// Returns an error if the JSON contains invalid syntax, missing required fields,
    /// or if an I/O error occurs while reading.
    pub fn from_json_reader<R: Read>(reader: R, options: &'a Options) -> Result<Self> {
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
    pub fn into_tally(self) -> Tally {
        self.tally
    }

    /// Gets the `tally` field.
    pub fn tally(&self) -> &[(Word, Count)] {
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
    pub const fn uniq_count(&self) -> Count {
        self.uniq_count
    }

    /// Gets the `count` field.
    pub const fn count(&self) -> Count {
        self.count
    }

    /// Sorts the `tally` field in place if a sort order other than `Unsorted` is provided.
    pub fn sort(&mut self, sort: Sort) {
        sort.apply(self);
    }

    /// Reads from input into a String, handling errors appropriately
    fn read_to_string<T: Read>(mut input: T, options: &Options) -> String {
        let mut content = String::new();
        if let Err(err) = input.read_to_string(&mut content) {
            if options.performance().verbose() {
                eprintln!("Warning: No content will be processed. Failed to read input: {err}");
            }
        }
        content
    }

    /// Counts words in a string and adds them to the tally map
    fn add_words_to_tally(tally: &mut TallyMap, line: &str, case: Case) {
        for word in line.unicode_words() {
            *tally.entry(case.normalize(word)).or_insert(0) += 1;
        }
    }

    /// Processes lines sequentially, adding words to the tally
    fn process_lines_sequentially<'b, I>(lines: I, options: &Options) -> TallyMap
    where
        I: Iterator<Item = &'b str>,
    {
        let mut tally = TallyMap::with_capacity(options.performance().estimate_capacity());
        let case = options.case();

        for line in lines {
            Self::add_words_to_tally(&mut tally, line, case);
        }

        tally
    }

    /// Merges maps by combining word counts
    fn merge_map_into(dest: &mut TallyMap, source: TallyMap) {
        for (word, count) in source {
            *dest.entry(word).or_insert(0) += count;
        }
    }

    /// Reserves capacity in the result map if needed based on dynamic threshold
    ///
    /// - Use a dynamic threshold based on input size and configuration
    /// - Only reserve capacity when the combined size exceeds the threshold
    /// - Avoid unnecessary allocations for small maps
    /// - Provide more aggressive capacity reservation for large maps
    fn reserve_capacity_if_needed(
        result_map: &mut TallyMap,
        local_map_len: usize,
        options: &Options,
    ) {
        // Skip reservation for empty or very small maps
        if local_map_len <= 4 {
            return;
        }

        let threshold = options.performance().calc_reserve_threshold();
        let result_len = result_map.len();

        if result_len == 0 {
            // For empty result maps, allocate with adequate capacity
            result_map.reserve(local_map_len);
        } else if result_len + local_map_len > threshold {
            // For larger merges, use more aggressive reservation to reduce rehashing
            let capacity_needed = if local_map_len > 1000 {
                // For very large maps, add extra capacity to reduce future reallocations
                (local_map_len * 5) / 4 // Add 25% extra capacity
            } else {
                local_map_len
            };

            result_map.reserve(capacity_needed);
        }
    }

    /// Sequential implementation for streamed word tallying
    fn tally_map_sequential_streamed<T: Read>(reader: BufReader<T>, options: &Options) -> TallyMap {
        // Process each line as it comes in
        let mut tally = TallyMap::with_capacity(options.performance().estimate_capacity());
        let case = options.case();

        // Process lines one at a time, never loading the entire file into memory
        for line in reader.lines().map_while(Result::ok) {
            Self::add_words_to_tally(&mut tally, &line, case);
        }

        tally
    }

    /// Parallel implementation for streamed processing
    ///
    /// This implementation uses a limited-memory sliding window approach:
    /// - Process the input in fixed-size chunks without accumulating the entire file
    /// - Use Rayon's parallel iterator for each chunk independently
    /// - Combine results using thread-local storage
    fn tally_map_parallel_streamed<T: Read>(reader: BufReader<T>, options: &Options) -> TallyMap {
        let chunk_bytes = options.performance().chunk_size().max(16_384) as usize;
        let perf = options.performance();

        let result_map_capacity = perf.default_capacity().max(1024);
        let mut result_map = TallyMap::with_capacity(result_map_capacity);

        let buffer_capacity = (chunk_bytes / 64).max(256);
        let mut buffer: Vec<String> = Vec::with_capacity(buffer_capacity);
        let mut bytes_read = 0;

        for line_result in reader.lines() {
            if let Ok(line) = line_result {
                let line_len = line.len();

                // Process when buffer reaches chunk size
                if bytes_read + line_len > chunk_bytes && !buffer.is_empty() {
                    // Process current batch in parallel
                    let local_map = Self::process_lines_in_parallel(&buffer, options);

                    // Merge results
                    Self::reserve_capacity_if_needed(&mut result_map, local_map.len(), options);
                    Self::merge_map_into(&mut result_map, local_map);

                    // Clear buffer for next batch
                    buffer.clear();
                    bytes_read = 0;
                }

                // Add current line to buffer
                buffer.push(line);
                bytes_read += line_len;
            } else {
                break;
            }
        }

        // Process any remaining lines
        if !buffer.is_empty() {
            let local_map = Self::process_lines_in_parallel(&buffer, options);

            // Merge final results
            Self::reserve_capacity_if_needed(&mut result_map, local_map.len(), options);
            Self::merge_map_into(&mut result_map, local_map);
        }

        result_map
    }

    /// Process a batch of lines in parallel using Rayon's work-stealing thread pool
    ///
    /// - Divide work into reasonably sized chunks to avoid per-item overhead
    /// - Use `fold()` for thread-local accumulation to minimize contention
    /// - Use `reduce()` for combining results
    fn process_lines_in_parallel(lines: &[String], options: &Options) -> TallyMap {
        let perf = options.performance();
        let case = options.case();

        let threads = match perf.threads() {
            Threads::All => rayon::current_num_threads(),
            Threads::Count(n) => n as usize,
        };

        let total_lines = lines.len();
        let thread_local_capacity = perf
            .estimate_thread_local_capacity()
            .min(total_lines / 2)
            .max(16);

        if total_lines < 32 || threads <= 1 {
            // For very small inputs, process sequentially to avoid parallelism overhead
            let mut result = TallyMap::with_capacity(thread_local_capacity);
            for line in lines {
                Self::add_words_to_tally(&mut result, line, case);
            }
            return result;
        }

        // Use Rayon's fold/reduce pattern for work stealing
        lines
            .par_iter()
            .fold(
                || TallyMap::with_capacity(thread_local_capacity),
                |mut acc, line| {
                    Self::add_words_to_tally(&mut acc, line, case);
                    acc
                },
            )
            .reduce(TallyMap::new, |mut a, mut b| {
                // Always merge smaller map into the larger one
                if a.len() < b.len() {
                    std::mem::swap(&mut a, &mut b);
                }

                // Pre-allocate capacity if the combined size is significant
                Self::reserve_capacity_if_needed(&mut a, b.len(), options);

                // Merge b into a
                for (word, count) in b {
                    *a.entry(word).or_insert(0) += count;
                }
                a
            })
    }

    /// Sequential implementation for buffered word tallying
    fn tally_map_sequential_buffered<T: Read>(input: T, options: &Options) -> TallyMap {
        let content = Self::read_to_string(input, options);
        let case = options.case();

        // Optimize map capacity using the actual content size
        let total_bytes = content.len();
        let uniqueness_ratio = options.performance().uniqueness_ratio() as usize;
        let estimated_capacity = total_bytes / uniqueness_ratio;

        let mut tally = TallyMap::with_capacity(estimated_capacity);

        // Process all lines in one efficient pass, leveraging being fully in memory
        for line in content.lines() {
            Self::add_words_to_tally(&mut tally, line, case);
        }

        tally
    }

    /// Parallel implementation for buffered word tallying
    ///
    /// - Read the entire input into memory once
    /// - Use Rayon's `par_lines()` for parallel processing
    /// - Provide capacity hints based on input size
    /// - Balance thread-local allocations for best performance
    /// - Use Rayon's fold/reduce pattern
    fn tally_map_parallel_buffered<T: Read>(input: T, options: &Options) -> TallyMap {
        let content = Self::read_to_string(input, options);
        let case = options.case();

        let perf = options.performance();
        let thread_capacity = perf.estimate_thread_local_capacity();
        let uniqueness_ratio = perf.uniqueness_ratio() as usize;
        let result_capacity = content.len() / uniqueness_ratio;

        // For small inputs, process sequentially to avoid parallelism overhead
        let line_count = content.lines().count();
        if line_count < 32 || matches!(perf.threads(), Threads::Count(1)) {
            let mut result = TallyMap::with_capacity(result_capacity.min(1024));
            for line in content.lines() {
                Self::add_words_to_tally(&mut result, line, case);
            }
            return result;
        }

        // Process content in parallel using Rayon's fold/reduce pattern
        content
            .par_lines()
            .fold(
                || TallyMap::with_capacity(thread_capacity),
                |mut local_map, line| {
                    Self::add_words_to_tally(&mut local_map, line, case);
                    local_map
                },
            )
            .reduce(TallyMap::new, |mut a, mut b| {
                // Always merge smaller map into larger one for efficiency
                if a.len() < b.len() {
                    std::mem::swap(&mut a, &mut b);
                }

                // Pre-allocate capacity if the combined size is significant
                Self::reserve_capacity_if_needed(&mut a, b.len(), options);

                // Merge b into a
                for (word, count) in b {
                    *a.entry(word).or_insert(0) += count;
                }
                a
            })
    }

    /// Process memory-mapped content in parallel using the OS virtual memory system
    ///
    /// This implementation leverages memory mapping for efficient file access:
    /// - Use the OS virtual memory system rather than loading content into RAM
    /// - Optimize capacity allocation based on file size
    /// - Use Rayon's recommended fold/reduce pattern for efficient parallelism
    fn process_mmap_parallel(content: &str, options: &Options) -> TallyMap {
        let perf = options.performance();
        let case = options.case();
        let thread_capacity = perf.estimate_thread_local_capacity();
        let result_capacity = content.len() / perf.uniqueness_ratio() as usize;

        // For small inputs, process sequentially to avoid parallelism overhead
        let line_count = content.lines().count();
        if line_count < 32 || matches!(perf.threads(), Threads::Count(1)) {
            let mut result = TallyMap::with_capacity(result_capacity.min(1024));
            for line in content.lines() {
                Self::add_words_to_tally(&mut result, line, case);
            }
            return result;
        }

        // Use Rayon's parallel iterator approach with memory-mapped content
        content
            .par_lines()
            .fold(
                || TallyMap::with_capacity(thread_capacity),
                |mut local_map, line| {
                    Self::add_words_to_tally(&mut local_map, line, case);
                    local_map
                },
            )
            .reduce(TallyMap::new, |mut a, mut b| {
                // Always merge smaller map into larger one for efficiency
                if a.len() < b.len() {
                    std::mem::swap(&mut a, &mut b);
                }

                // Pre-allocate capacity if the combined size is significant
                Self::reserve_capacity_if_needed(&mut a, b.len(), options);

                // Merge b into a
                for (word, count) in b {
                    *a.entry(word).or_insert(0) += count;
                }
                a
            })
    }

    fn tally_map_memory_mapped(file: &File, options: &Options) -> Result<TallyMap> {
        // Memory-mapping uses OS virtual memory for efficient file access
        // Note: All memory mapping operations require unsafe per the memmap2 crate design
        let mmap = unsafe { Mmap::map(file).context("Failed to memory map the file")? };
        let content =
            std::str::from_utf8(&mmap).context("Memory-mapped file contains invalid UTF-8")?;

        // Use the processing strategy from options
        let result = match options.processing() {
            Processing::Sequential => {
                // Stream through lines in the memory-mapped file sequentially
                Self::process_lines_sequentially(content.lines(), options)
            }
            Processing::Parallel => {
                // Process memory-mapped content in parallel
                Self::process_mmap_parallel(content, options)
            }
        };

        Ok(result)
    }
}
