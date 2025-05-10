//! Configuration options for word tallying.
//!
//! This module provides the [`Options`] struct, a unified container for all
//! word-tally configuration settings.
//!
//! # Structure
//!
//! Options are organized into three components:
//!
//! 1. **Formatting** ([`Formatting`]): Controls text normalization and presentation
//!    - [`Case`]: Word case handling (original, lowercase, uppercase)
//!    - [`Sort`]: Result ordering (unsorted, ascending, descending)
//!    - [`Format`]: Output format (text, CSV, JSON)
//!
//! 2. **Filters** ([`Filters`]): Determines which words appear in results
//!    - Length filters: Minimum character count
//!    - Frequency filters: Occurrence thresholds
//!    - Pattern matching: Regular expression filters
//!    - Word lists: Explicit exclusions
//!
//! 3. **Performance** ([`Performance`]): Optimizes processing efficiency
//!    - [`Processing`]: Processing mode (sequential, parallel)
//!    - [`Io`]: I/O method (streamed, buffered, memory-mapped)
//!    - [`Threads`]: Thread pool configuration
//!    - [`SizeHint`]: Collection allocation tuning
//!
//! # Usage
//!
//! ```
//! use word_tally::{Options, Case, Format, Io, Processing};
//!
//! // Default options
//! let options = Options::default();
//!
//! // With specific settings
//! let options = Options::default()
//!     .with_case(Case::Lower)
//!     .with_format(Format::Json)
//!     .with_processing(Processing::Parallel)
//!     .with_io(Io::MemoryMapped);
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

use crate::filters::Filters;
use crate::formatting::{Case, Format, Formatting, Sort};
use crate::performance::{Io, Performance, Processing, SizeHint, Threads};
use core::fmt;
use serde::{Deserialize, Serialize};

/// Unified configuration for word tallying operations.
///
/// `Options` consolidates all configuration aspects of word tallying into a single structure.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub struct Options {
    /// Formatting configuration (case normalization, sorting, output format)
    formatting: Formatting,

    /// Filter settings (word length, frequency, patterns, exclusions)
    filters: Filters,

    /// Performance configuration (I/O strategy, processing mode, threads, memory allocation)
    performance: Performance,
}

impl Options {
    /// Creates a new `Options` with custom formatting, filters, and performance configurations.
    ///
    /// # Examples
    ///
    /// ```
    /// use word_tally::{Options, Formatting, Filters, Performance, Case, Format, Io, Processing};
    ///
    /// // Customization using component structs
    /// let options = Options::new(
    ///     Formatting::default().with_case_setting(Case::Lower),
    ///     Filters::default().with_min_chars(3),
    ///     Performance::default().with_processing(Processing::Parallel)
    /// );
    ///
    /// // Default configuration
    /// let options = Options::default();
    ///
    /// // Targeted customization with builder methods
    /// let options = Options::default()
    ///     .with_case(Case::Lower)
    ///     .with_format(Format::Json);
    /// ```
    ///
    /// # Tests
    ///
    /// ```
    /// use word_tally::{Options, Formatting, Filters, Performance, Case, Format, Processing};
    ///
    /// // Verify examples work correctly
    /// let custom_options = Options::new(
    ///     Formatting::default().with_case_setting(Case::Lower),
    ///     Filters::default().with_min_chars(3),
    ///     Performance::default().with_processing(Processing::Parallel)
    /// );
    ///
    /// let default_options = Options::default();
    ///
    /// let builder_options = Options::default()
    ///     .with_case(Case::Lower)
    ///     .with_format(Format::Json);
    ///
    /// assert_eq!(custom_options.formatting().case(), Case::Lower);
    /// assert!(custom_options.filters().min_chars().is_some());
    /// assert_eq!(custom_options.processing(), Processing::Parallel);
    ///
    /// assert_eq!(builder_options.formatting().case(), Case::Lower);
    /// assert_eq!(builder_options.formatting().format(), Format::Json);
    /// ```
    pub const fn new(formatting: Formatting, filters: Filters, performance: Performance) -> Self {
        Self {
            formatting,
            filters,
            performance,
        }
    }

    /// Create a new Options instance with default filters
    pub fn with_defaults(formatting: Formatting, performance: Performance) -> Self {
        Self::new(formatting, Filters::default(), performance)
    }

    /// Create Options with default Filters and Performance
    pub fn from_formatting(formatting: Formatting) -> Self {
        Self::with_defaults(formatting, Performance::default())
    }

    /// Create Options with default Formatting and Filters
    pub fn from_performance(performance: Performance) -> Self {
        Self::with_defaults(Formatting::default(), performance)
    }

    /// Set formatting options while preserving other options
    pub const fn with_formatting(mut self, formatting: Formatting) -> Self {
        self.formatting = formatting;
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

    /// Set case normalization option
    pub const fn with_case(mut self, case: Case) -> Self {
        self.formatting = self.formatting.with_case_setting(case);
        self
    }

    /// Set sort order option
    pub const fn with_sort(mut self, sort: Sort) -> Self {
        self.formatting = self.formatting.with_sort_setting(sort);
        self
    }

    /// Set output format while preserving other options
    pub const fn with_format(mut self, format: Format) -> Self {
        self.formatting = self.formatting.with_format_setting(format);
        self
    }

    /// Set I/O strategy
    pub const fn with_io(mut self, io: Io) -> Self {
        self.performance = self.performance.with_io(io);
        self
    }

    /// Set processing strategy
    pub const fn with_processing(mut self, processing: Processing) -> Self {
        self.performance = self.performance.with_processing(processing);
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

    /// Set default capacity for collections
    pub const fn with_capacity(mut self, capacity: usize) -> Self {
        self.performance = self.performance.with_capacity(capacity);
        self
    }

    /// Set uniqueness ratio for capacity estimation
    pub const fn with_uniqueness_ratio(mut self, ratio: u8) -> Self {
        self.performance = self.performance.with_uniqueness_ratio(ratio);
        self
    }

    /// Set word density for chunk capacity estimation
    pub const fn with_word_density(mut self, density: u8) -> Self {
        self.performance = self.performance.with_word_density(density);
        self
    }

    /// Set chunk size for parallel processing
    pub const fn with_chunk_size(mut self, size: usize) -> Self {
        self.performance = self.performance.with_chunk_size(size);
        self
    }

    /// Get a reference to the formatting options
    pub const fn formatting(&self) -> &Formatting {
        &self.formatting
    }

    /// Get a reference to the filters
    pub const fn filters(&self) -> &Filters {
        &self.filters
    }

    /// Get a reference to the performance configuration
    pub const fn performance(&self) -> &Performance {
        &self.performance
    }

    /// Get the case setting from formatting options
    pub const fn case(&self) -> Case {
        self.formatting.case()
    }

    /// Get the sort setting from formatting options
    pub const fn sort(&self) -> Sort {
        self.formatting.sort()
    }

    /// Get the format setting from formatting options
    pub const fn format(&self) -> Format {
        self.formatting.format()
    }

    /// Get the I/O strategy from performance options
    pub const fn io(&self) -> Io {
        self.performance.io()
    }

    /// Get the processing strategy from performance options
    pub const fn processing(&self) -> Processing {
        self.performance.processing()
    }

    /// Get the threads setting from performance options
    pub const fn threads(&self) -> Threads {
        self.performance.threads()
    }

    /// Get the size hint from performance options
    pub const fn size_hint(&self) -> SizeHint {
        self.performance.size_hint()
    }

    /// Estimate capacity using performance configuration
    pub const fn estimate_capacity(&self) -> usize {
        self.performance.estimate_capacity()
    }

    /// Estimate chunk capacity using performance configuration
    pub const fn estimate_chunk_capacity(&self, chunk_size: usize) -> usize {
        self.performance.estimate_chunk_capacity(chunk_size)
    }
}

impl fmt::Display for Options {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Options {{ formatting: {}, filters: {:?}, processing: {:?}, io: {:?} }}",
            self.formatting,
            self.filters,
            self.performance.processing(),
            self.performance.io()
        )
    }
}

impl From<(Formatting, Filters, Performance)> for Options {
    fn from((formatting, filters, performance): (Formatting, Filters, Performance)) -> Self {
        Self::new(formatting, filters, performance)
    }
}

impl From<(Formatting, Performance)> for Options {
    fn from((formatting, performance): (Formatting, Performance)) -> Self {
        Self::with_defaults(formatting, performance)
    }
}

impl From<Formatting> for Options {
    fn from(formatting: Formatting) -> Self {
        Self::with_defaults(formatting, Performance::default())
    }
}

impl From<Performance> for Options {
    fn from(performance: Performance) -> Self {
        Self::with_defaults(Formatting::default(), performance)
    }
}

impl AsRef<Formatting> for Options {
    fn as_ref(&self) -> &Formatting {
        &self.formatting
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
