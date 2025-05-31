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
//! - **Io** ([`Io`]): I/O strategy (sequential, streamed, in-memory, memory-mapped)
//! - **Performance** ([`Performance`]): Thread pool, memory allocation, and chunk size tuning
//!
//! # Usage
//!
//! ```
//! use word_tally::{Options, Case, Format, Io, Serialization};
//!
//! // Default options
//! let options = Options::default();
//! assert_eq!(options.case(), Case::Original);
//!
//! // With specific settings
//! let options = Options::default()
//!     .with_case(Case::Lower)
//!     .with_serialization(Serialization::with_format(Format::Json))
//!     .with_io(Io::ParallelMmap);
//! assert_eq!(options.io(), Io::ParallelMmap);
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

pub mod case;
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
use crate::WordTallyError;
use anyhow::Result;
use core::fmt::{self, Display, Formatter};
use serde::{Deserialize, Serialize};

/// Unified configuration for word tallying operations.
///
/// `Options` consolidates all configuration aspects of word tallying into a single structure.
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
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

    /// Word encoding strategy (unicode, ascii).
    encoding: Encoding,
}

impl Options {
    /// Creates a new `Options` with custom case, sort, serializer, filters, and performance configurations.
    ///
    /// # Examples
    ///
    /// ```
    /// use word_tally::{Options, Serialization, Filters, Performance, Case, Format, Io, Sort};
    ///
    /// // Default configuration
    /// let options = Options::default();
    /// assert_eq!(options.io(), Io::ParallelStream);
    ///
    /// // Targeted customization with builder methods
    /// let options = Options::default()
    ///     .with_case(Case::Lower)
    ///     .with_serialization(Serialization::with_format(Format::Json));
    /// assert_eq!(options.serialization().format(), Format::Json);
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

    /// Set word encoding strategy.
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

    /// Get the word encoding strategy.
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

/// Unescape a string, converting escape sequences like `\t` and `\n` to their actual characters.
///
/// Used for parsing delimiters and exclude words from command-line arguments.
///
/// # Errors
///
/// Returns an error if the input contains a trailing backslash without a character to escape.
pub fn unescape(input: &str, context: &str) -> Result<String> {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars();

    while let Some(ch) = chars.next() {
        match ch {
            '\\' => match chars.next() {
                Some('t') => result.push('\t'),
                Some('n') => result.push('\n'),
                Some('r') => result.push('\r'),
                Some('"') => result.push('"'),
                Some('\'') => result.push('\''),
                Some('\\') => result.push('\\'),
                Some(other) => result.push(other),
                None => {
                    return Err(WordTallyError::Unescape {
                        context: context.to_string(),
                        value: input.to_string(),
                    }
                    .into());
                }
            },
            _ => result.push(ch),
        }
    }

    Ok(result)
}
