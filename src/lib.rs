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
//! - `exit_code.rs`: Exit code definitions and handling
//! - `input.rs`: Input source management strategies
//! - `lib.rs`: Core library functionality and API
//! - `main.rs`: CLI entry point and execution
//! - `options/`: Configuration and processing options
//!   - `options/case.rs`: Text case normalization utilities
//!   - `options/filters.rs`: Word filtering mechanisms
//!   - `options/io.rs`: I/O operations implementation
//!   - `options/mod.rs`: Common options functionality
//!   - `options/patterns.rs`: Regular expression and pattern matching
//!   - `options/performance.rs`: Optimization and benchmarking
//!   - `options/processing.rs`: Text processing algorithms
//!   - `options/serialization.rs`: Data serialization for exports
//!   - `options/sort.rs`: Word frequency sorting strategies
//!   - `options/threads.rs`: Thread configuration
//! - `output.rs`: Output formatting and display
//! - `tally_map.rs`: Map for tallying word counts
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

use std::{hash::Hash, str};

use anyhow::Result;
use serde::{self, Deserialize, Serialize};

pub mod exit_code;
pub mod input;
pub mod options;
pub mod output;
pub mod tally_map;

pub use input::{Input, InputReader};
pub use options::patterns::{ExcludePatterns, IncludePatterns, InputPatterns};
pub use options::{
    Options,
    case::Case,
    filters::{ExcludeWords, Filters, MinChars, MinCount},
    io::Io,
    performance::Performance,
    processing::Processing,
    serialization::{Format, Serialization},
    sort::Sort,
    threads::Threads,
};
pub use output::Output;
pub use tally_map::TallyMap;

pub type Count = usize;
pub type Word = Box<str>;
pub type Tally = Box<[(Word, Count)]>;

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
            options.performance().threads.init_pool()?;
        }

        // Generate the tally map from the input
        let tally_map = TallyMap::from_input(input, options)?;

        Ok(Self::from_tally_map(tally_map, options))
    }

    /// Creates a `WordTally` instance from a `TallyMap` and `Options`.
    fn from_tally_map(mut tally_map: TallyMap, options: &'a Options) -> Self {
        options.filters().apply(&mut tally_map, options.case());

        let count = tally_map.values().sum();
        let tally = tally_map.into_tally();
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
}
