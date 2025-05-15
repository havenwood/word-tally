//! Command-line argument parsing and access.

use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;

use word_tally::options::{
    case::Case,
    filters::Filters,
    io::Io,
    performance::Performance,
    processing::{Processing, SizeHint},
    serialization::{Format, Serialization},
    sort::Sort,
};
use word_tally::{Count, Options};

/// A utility for tallying word frequencies in text.
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Parser)]
#[command(
    name = "word-tally",
    author,
    version,
    about,
    long_about = "Tally word frequencies with customizable options for sorting, filtering, and output formatting"
)]
pub struct Args {
    /// File path to use as input rather than stdin ("-").
    #[arg(default_value = "-", value_name = "PATH")]
    input: String,

    // Performance options
    /// I/O strategy.
    #[arg(short = 'I', long, value_enum, default_value_t = Io::Streamed, value_name = "STRATEGY")]
    io: Io,

    /// Use threads for parallel processing.
    #[arg(short = 'p', long)]
    parallel: bool,

    // Output formatting options
    /// Case normalization.
    #[arg(short, long, default_value_t, value_enum, value_name = "FORMAT")]
    case: Case,

    /// Sort order.
    #[arg(short, long, default_value_t, value_enum, value_name = "ORDER")]
    sort: Sort,

    // Filtering options
    /// Exclude words containing fewer than min chars.
    #[arg(short, long, value_name = "COUNT")]
    min_chars: Option<Count>,

    /// Exclude words appearing fewer than min times.
    #[arg(short = 'M', long, value_name = "COUNT")]
    min_count: Option<Count>,

    /// Exclude words from a comma-delimited list.
    #[arg(short = 'E', long, use_value_delimiter = true, value_name = "WORDS")]
    exclude_words: Option<Vec<String>>,

    /// Include only words matching a regex pattern.
    #[arg(short = 'i', long, value_name = "PATTERN", action = clap::ArgAction::Append)]
    include: Option<Vec<String>>,

    /// Exclude words matching a regex pattern.
    #[arg(short = 'x', long, value_name = "PATTERN", action = clap::ArgAction::Append)]
    exclude: Option<Vec<String>>,

    // Output options
    /// Output format.
    #[arg(short = 'f', long, default_value_t, value_enum, value_name = "FORMAT")]
    format: Format,

    /// Delimiter between keys and values.
    #[arg(short, long, default_value = " ", value_name = "VALUE")]
    delimiter: String,

    /// Write output to file rather than stdout.
    #[arg(short, long, value_name = "PATH")]
    output: Option<PathBuf>,

    /// Print verbose details.
    #[arg(short = 'v', long)]
    verbose: bool,
}

impl Args {
    /// Get input file path
    #[expect(
        clippy::missing_const_for_fn,
        reason = "Blocked by const limitations until Rust 1.87.0"
    )]
    // Make this const when `const_vec_string_slice` is fully stabilized.
    pub fn get_input(&self) -> &str {
        &self.input
    }

    /// Get output file path
    pub const fn get_output(&self) -> &Option<PathBuf> {
        &self.output
    }

    /// Get verbose flag
    pub const fn is_verbose(&self) -> bool {
        self.verbose
    }

    /// Parse command-line arguments and convert them to word-tally `Options`.
    pub fn get_options(&self, size_hint: SizeHint) -> Result<Options> {
        let serialization = Serialization::new(self.format, &self.delimiter)?;
        let filters = self.get_filters()?;

        // Determine processing mode from the --parallel flag
        let processing = if self.parallel {
            Processing::Parallel
        } else {
            Processing::Sequential
        };

        let performance = Performance::default()
            .with_size_hint(size_hint)
            .with_verbose(self.verbose);

        // Create Options with case, sort, serialization, filters, io, processing, and performance
        Ok(Options::new(
            self.case,
            self.sort,
            serialization,
            filters,
            self.io,
            processing,
            performance,
        ))
    }

    /// Helper to create filters from arguments
    fn get_filters(&self) -> Result<Filters> {
        let mut filters = Filters::default();

        if let Some(min_chars) = self.min_chars {
            filters = filters.with_min_chars(min_chars);
        }

        if let Some(min_count) = self.min_count {
            filters = filters.with_min_count(min_count);
        }

        if let Some(words) = &self.exclude_words {
            filters = filters.with_unescaped_exclude_words(words)?;
        }

        if let Some(exclude_patterns) = &self.exclude {
            filters = filters
                .with_exclude_patterns(exclude_patterns)
                .with_context(|| "Failed to create exclude patterns")?;
        }

        if let Some(include_patterns) = &self.include {
            filters = filters
                .with_include_patterns(include_patterns)
                .with_context(|| "Failed to create include patterns")?;
        }

        Ok(filters)
    }
}
