use crate::filters::Filters;
use crate::formatting::{Case, Format, Formatting, Sort};
use crate::performance::{Concurrency, Performance, SizeHint, Threads};
use core::fmt;
use serde::{Deserialize, Serialize};

/// Unified options for word tallying, combining formatting, filtering, and performance settings.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub struct Options {
    /// Options for word formatting, case normalization, sorting, and output format
    formatting: Formatting,

    /// Filters for which words should be tallied
    filters: Filters,

    /// Configuration for performance, processing strategies, and resource allocation
    performance: Performance,
}

impl Options {
    /// Create a new unified Options with custom formatting, filters, and performance configurations
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

    // Specific formatting builder methods

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

    // Specific performance builder methods

    /// Set concurrency mode
    pub fn with_concurrency(mut self, concurrency: Concurrency) -> Self {
        self.performance = self.performance.with_concurrency(concurrency);
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
    pub const fn with_chunk_size(mut self, size: u32) -> Self {
        self.performance = self.performance.with_chunk_size(size);
        self
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

    /// Set output format while preserving other options
    pub const fn with_format(mut self, format: Format) -> Self {
        self.formatting = self.formatting.with_format_setting(format);
        self
    }

    /// Get the concurrency setting from performance options
    pub const fn concurrency(&self) -> Concurrency {
        self.performance.concurrency()
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

// From<(Formatting, Filters, Performance)> for Options
impl From<(Formatting, Filters, Performance)> for Options {
    fn from((formatting, filters, performance): (Formatting, Filters, Performance)) -> Self {
        Self::new(formatting, filters, performance)
    }
}

// From<(Formatting, Performance)> for Options with default Filters
impl From<(Formatting, Performance)> for Options {
    fn from((formatting, performance): (Formatting, Performance)) -> Self {
        Self::with_defaults(formatting, performance)
    }
}

// From<Formatting> for Options with default Filters and Performance
impl From<Formatting> for Options {
    fn from(formatting: Formatting) -> Self {
        Self::with_defaults(formatting, Performance::default())
    }
}

// From<Performance> for Options with default Formatting and Filters
impl From<Performance> for Options {
    fn from(performance: Performance) -> Self {
        Self::with_defaults(Formatting::default(), performance)
    }
}

// Implement AsRef for Options components for better interoperability
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

impl fmt::Display for Options {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Options {{ formatting: {}, filters: {:?}, concurrency: {:?} }}",
            self.formatting,
            self.filters,
            self.performance.concurrency()
        )
    }
}
