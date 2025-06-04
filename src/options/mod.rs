//! Configuration options for word tallying.
//!
//! The [`Options`] struct provides a builder API for configuring word tallying behavior.
//!
//! # Common Patterns
//!
//! ```
//! use word_tally::{Options, Case, Io, Serialization, Filters};
//!
//! // Fast processing of large files
//! let fast = Options::default()
//!     .with_io(Io::ParallelMmap);
//!
//! // Memory-constrained environment
//! let low_memory = Options::default()
//!     .with_io(Io::Stream);
//!
//! // Case-insensitive frequency analysis
//! let frequency = Options::default()
//!     .with_case(Case::Lower)
//!     .with_filters(Filters::default().with_min_count(2));
//!
//! // Export for data analysis
//! let export = Options::default()
//!     .with_serialization(Serialization::Csv)
//!     .with_filters(Filters::default().with_min_chars(4));
//! ```
//!
//! # Components
//!
//! - [`Case`] - Word case normalization
//! - [`encoding::Encoding`] - Text encoding validation and word detection
//! - [`Sort`] - Result ordering
//! - [`Serialization`] - Output format
//! - [`Filters`] - Word filtering rules
//! - [`Io`] - I/O strategy
//! - [`Performance`] - Performance tuning
//!
//! # Environment Variables
//!
//! - `WORD_TALLY_IO` (default: `parallel-stream`)
//! - `WORD_TALLY_THREADS` (default: all cores)
//! - `WORD_TALLY_CHUNK_SIZE` (default: 65536)
//! - `WORD_TALLY_UNIQUENESS_RATIO` (default: 32)
//! - `WORD_TALLY_WORDS_PER_KB` (default: 128)
//! - `WORD_TALLY_STDIN_BUFFER_SIZE` (default: 262144)

pub mod case;
pub mod delimiter;
pub mod encoding;
pub mod filters;
pub mod io;
pub mod patterns;
pub mod performance;
pub mod serialization;
pub mod sort;
pub mod threads;

use self::case::Case;
use self::encoding::Encoding;
use self::filters::Filters;
use self::io::Io;
use self::performance::Performance;
use self::serialization::Serialization;
use self::sort::Sort;
use anyhow::Result;
use core::fmt::{self, Display, Formatter};
use serde::{Deserialize, Serialize};

/// Unified configuration for word tallying operations.
///
/// `Options` consolidates all configuration aspects of word tallying into a single structure.
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Options {
    /// Case handling strategy (original, lower, upper).
    case: Case,

    /// Sort order for results (unsorted, ascending, descending).
    sort: Sort,

    /// Serialization configuration (output format, delimiter).
    serialization: Serialization,

    /// Filter settings (word length, frequency, patterns, exclusions).
    filters: Filters,

    /// I/O strategy (sequential, streamed, in-memory, memory-mapped).
    io: Io,

    /// Performance tuning configuration (threads, memory allocation, chunk size).
    performance: Performance,

    /// Text encoding strategy (unicode, ascii).
    encoding: Encoding,
}

impl Options {
    /// Creates a new `Options` with custom case, sort, serializer, filters, and performance configurations.
    ///
    /// # Examples
    ///
    /// ```
    /// use word_tally::{Options, Serialization, Filters, Performance, Case, Io, Sort};
    ///
    /// // Default configuration
    /// let options = Options::default();
    /// assert_eq!(options.io(), Io::ParallelStream);
    ///
    /// // Targeted customization with builder methods
    /// let options = Options::default()
    ///     .with_case(Case::Lower)
    ///     .with_serialization(Serialization::Json);
    /// assert_eq!(options.serialization(), &Serialization::Json);
    /// ```
    #[must_use]
    pub const fn new(
        case: Case,
        sort: Sort,
        serialization: Serialization,
        filters: Filters,
        io: Io,
        performance: Performance,
    ) -> Self {
        Self {
            case,
            sort,
            serialization,
            filters,
            io,
            performance,
            encoding: Encoding::Unicode,
        }
    }

    /// Set case handling strategy.
    #[must_use]
    pub const fn with_case(mut self, case: Case) -> Self {
        self.case = case;
        self
    }

    /// Set sort order.
    #[must_use]
    pub const fn with_sort(mut self, sort: Sort) -> Self {
        self.sort = sort;
        self
    }

    /// Set serialization options while preserving other options.
    #[must_use]
    pub fn with_serialization(mut self, serialization: Serialization) -> Self {
        self.serialization = serialization;
        self
    }

    /// Set filters while preserving other options.
    #[must_use]
    pub fn with_filters(mut self, filters: Filters) -> Self {
        self.filters = filters;
        self
    }

    /// Set performance configuration while preserving other options.
    #[must_use]
    pub const fn with_performance(mut self, performance: Performance) -> Self {
        self.performance = performance;
        self
    }

    /// Set I/O strategy.
    #[must_use]
    pub const fn with_io(mut self, io: Io) -> Self {
        self.io = io;
        self
    }

    /// Set text encoding strategy.
    #[must_use]
    pub const fn with_encoding(mut self, encoding: Encoding) -> Self {
        self.encoding = encoding;
        self
    }

    /// Get the case normalization setting.
    #[must_use]
    pub const fn case(&self) -> Case {
        self.case
    }

    /// Get the word sorting setting.
    #[must_use]
    pub const fn sort(&self) -> Sort {
        self.sort
    }

    /// Get a reference to the serialization options.
    #[must_use]
    pub const fn serialization(&self) -> &Serialization {
        &self.serialization
    }

    /// Get a reference to the filters.
    #[must_use]
    pub const fn filters(&self) -> &Filters {
        &self.filters
    }

    /// Get a reference to the performance configuration.
    #[must_use]
    pub const fn performance(&self) -> &Performance {
        &self.performance
    }

    /// Get the I/O strategy.
    #[must_use]
    pub const fn io(&self) -> Io {
        self.io
    }

    /// Get the text encoding strategy.
    #[must_use]
    pub const fn encoding(&self) -> Encoding {
        self.encoding
    }

    /// Initialize the thread pool if using a parallel I/O mode.
    ///
    /// This method initializes the global thread pool when using parallel I/O modes
    /// (streamed, in-memory, or memory-mapped). For sequential mode, this is a no-op.
    ///
    /// # Errors
    ///
    /// Returns an error if a parallel I/O mode is selected but the thread pool
    /// cannot be initialized.
    pub fn init_thread_pool_if_parallel(&self) -> Result<()> {
        match self.io {
            Io::Stream => Ok(()),
            Io::ParallelStream | Io::ParallelInMemory | Io::ParallelMmap | Io::ParallelBytes => {
                self.performance.threads().init_pool()
            }
        }
    }
}

impl Display for Options {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Options {{ case: {}, sort: {}, serialization: {}, filters: {:?}, io: {}, encoding: {} }}",
            self.case, self.sort, self.serialization, self.filters, self.io, self.encoding
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
