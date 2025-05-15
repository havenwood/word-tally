//! Configuration options for word tallying.
//!
//! This module provides the [`Options`] struct, a unified container for all
//! word-tally configuration settings.
//!
//! # Structure
//!
//! Core configuration components:
//!
//! - **Case** ([`Case`]): Word case handling (original, lowercase, uppercase)
//! - **Sort** ([`Sort`]): Result ordering (unsorted, ascending, descending)
//! - **Serialization** ([`Serialization`]): Output format (text, CSV, JSON) and delimiter
//! - **Filters** ([`Filters`]): Word length, frequency, patterns, and exclusion filters
//! - **Io** ([`Io`]): I/O strategy (streamed, buffered, memory-mapped)
//! - **Processing** ([`Processing`]): Processing mode (sequential, parallel)
//! - **Performance** ([`Performance`]): Thread pool, memory allocation, and chunk size tuning
//!
//! # Usage
//!
//! ```
//! use word_tally::{Options, Case, Format, Io, Processing};
//!
//! // Default options
//! let options = Options::default();
//! assert_eq!(options.case(), Case::Lower);
//!
//! // With specific settings
//! let options = Options::default()
//!     .with_case(Case::Lower)
//!     .with_format(Format::Json)
//!     .with_processing(Processing::Parallel)
//!     .with_io(Io::MemoryMapped);
//! assert_eq!(options.io(), Io::MemoryMapped);
//! ```
//!
//! # Environment Variables
//!
//! Performance settings can be controlled via environment variables:
//!
//! - `WORD_TALLY_CHUNK_SIZE`: Chunk size for parallel processing (default: 16384)
//! - `WORD_TALLY_THREADS`: Thread count (default: all available cores)
//! - `WORD_TALLY_UNIQUENESS_RATIO`: Capacity estimation (default: 10)
//! - `WORD_TALLY_WORD_DENSITY`: Per-chunk map capacity (default: 15)

use crate::case::Case;
use crate::filters::Filters;
use crate::io::Io;
use crate::performance::Performance;
use crate::processing::{Processing, SizeHint, Threads};
use crate::serialization::Format;
use crate::serialization::Serialization;
use crate::sort::Sort;
use core::fmt;
use serde::{Deserialize, Serialize};

/// Unified configuration for word tallying operations.
///
/// `Options` consolidates all configuration aspects of word tallying into a single structure.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default, Hash)]
pub struct Options {
    /// Case handling strategy (original, lower, upper)
    case: Case,

    /// Sort order for results (unsorted, ascending, descending)
    sort: Sort,

    /// Serialization configuration (output format, delimiter)
    serialization: Serialization,

    /// Filter settings (word length, frequency, patterns, exclusions)
    filters: Filters,

    /// I/O strategy (streamed, buffered, memory-mapped)
    io: Io,

    /// Processing strategy (sequential, parallel)
    processing: Processing,

    /// Performance tuning configuration (threads, memory allocation, chunk size)
    performance: Performance,
}

impl Options {
    /// Creates a new `Options` with custom case, sort, serializer, filters, and performance configurations.
    ///
    /// # Examples
    ///
    /// ```
    /// use word_tally::{Options, Serialization, Filters, Performance, Case, Format, Io, Processing, Sort};
    ///
    /// // Default configuration
    /// let options = Options::default();
    /// assert_eq!(options.processing(), Processing::Sequential);
    ///
    /// // Targeted customization with builder methods
    /// let options = Options::default()
    ///     .with_case(Case::Lower)
    ///     .with_format(Format::Json);
    /// assert_eq!(options.serialization().format(), Format::Json);
    /// ```
    pub const fn new(
        case: Case,
        sort: Sort,
        serialization: Serialization,
        filters: Filters,
        io: Io,
        processing: Processing,
        performance: Performance,
    ) -> Self {
        Self {
            case,
            sort,
            serialization,
            filters,
            io,
            processing,
            performance,
        }
    }

    /// Create a new Options instance with default filters
    pub fn with_defaults(
        case: Case,
        sort: Sort,
        serialization: Serialization,
        io: Io,
        processing: Processing,
        performance: Performance,
    ) -> Self {
        Self::new(
            case,
            sort,
            serialization,
            Filters::default(),
            io,
            processing,
            performance,
        )
    }

    /// Set case handling strategy
    pub const fn with_case(mut self, case: Case) -> Self {
        self.case = case;
        self
    }

    /// Set sort order
    pub const fn with_sort(mut self, sort: Sort) -> Self {
        self.sort = sort;
        self
    }

    /// Set serialization options while preserving other options
    pub fn with_serialization(mut self, serialization: Serialization) -> Self {
        self.serialization = serialization;
        self
    }

    /// Set filters while preserving other options
    pub fn with_filters(mut self, filters: Filters) -> Self {
        self.filters = filters;
        self
    }

    /// Set performance configuration while preserving other options
    pub const fn with_performance(mut self, performance: Performance) -> Self {
        self.performance = performance;
        self
    }

    /// Set output format while preserving other options
    pub fn with_format(mut self, format: Format) -> Self {
        self.serialization = self.serialization.with_format_setting(format);
        self
    }

    /// Set delimiter for text output
    pub fn with_delimiter(mut self, delimiter: String) -> Self {
        let mut serialization = self.serialization;
        serialization.delimiter = delimiter;
        self.serialization = serialization;
        self
    }

    /// Set I/O strategy
    pub const fn with_io(mut self, io: Io) -> Self {
        self.io = io;
        self
    }

    /// Set processing strategy
    pub const fn with_processing(mut self, processing: Processing) -> Self {
        self.processing = processing;
        self
    }

    /// Set size hint for capacity optimization
    pub const fn with_size_hint(mut self, size_hint: SizeHint) -> Self {
        self.performance = self.performance.with_size_hint(size_hint);
        self
    }

    /// Set thread count for parallel processing
    pub const fn with_threads(mut self, threads: Threads) -> Self {
        self.performance = self.performance.with_threads(threads);
        self
    }

    /// Set default capacity for TallyMap
    pub const fn with_capacity(mut self, capacity: usize) -> Self {
        self.performance = self.performance.with_tally_map_capacity(capacity);
        self
    }

    /// Set uniqueness ratio for capacity estimation
    pub const fn with_uniqueness_ratio(mut self, ratio: u8) -> Self {
        self.performance = self.performance.with_uniqueness_ratio(ratio);
        self
    }

    /// Set words-per-kilobyte for capacity estimation
    pub const fn with_words_per_kb(mut self, words_per_kb: u8) -> Self {
        self.performance = self.performance.with_words_per_kb(words_per_kb);
        self
    }

    /// Set chunk size for parallel processing
    pub const fn with_chunk_size(mut self, size: usize) -> Self {
        self.performance = self.performance.with_chunk_size(size);
        self
    }

    /// Get the case normalization setting
    pub const fn case(&self) -> Case {
        self.case
    }

    /// Get the word sorting setting
    pub const fn sort(&self) -> Sort {
        self.sort
    }

    /// Get a reference to the serialization options
    pub const fn serialization(&self) -> &Serialization {
        &self.serialization
    }

    /// Get a reference to the filters
    pub const fn filters(&self) -> &Filters {
        &self.filters
    }

    /// Get a reference to the performance configuration
    pub const fn performance(&self) -> &Performance {
        &self.performance
    }

    /// Get the I/O strategy
    pub const fn io(&self) -> Io {
        self.io
    }

    /// Get the processing strategy
    pub const fn processing(&self) -> Processing {
        self.processing
    }
}

impl fmt::Display for Options {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Options {{ case: {}, sort: {}, serialization: {}, filters: {:?}, processing: {}, io: {} }}",
            self.case, self.sort, self.serialization, self.filters, self.processing, self.io
        )
    }
}

impl AsRef<Serialization> for Options {
    fn as_ref(&self) -> &Serialization {
        &self.serialization
    }
}

impl AsRef<Filters> for Options {
    fn as_ref(&self) -> &Filters {
        &self.filters
    }
}

impl AsRef<Performance> for Options {
    fn as_ref(&self) -> &Performance {
        &self.performance
    }
}
