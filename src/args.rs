//! Command-line argument parsing and access.

use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;
use unescaper::unescape;

use word_tally::filters::Filters;
use word_tally::formatting::{Case, Format, Formatting, Sort};
use word_tally::options::Options;
use word_tally::performance::{Concurrency, Performance, SizeHint};

/// A utility for tallying word frequencies in text.
#[derive(Debug, Parser)]
#[command(about, version)]
pub struct Args {
    /// File path to use as input rather than stdin ("-").
    #[arg(default_value = "-", value_name = "PATH")]
    input: String,

    // Formatting options
    /// Case normalization.
    #[arg(short, long, default_value_t, value_enum, value_name = "FORMAT")]
    case: Case,

    /// Sort order.
    #[arg(short, long, default_value_t, value_enum, value_name = "ORDER")]
    sort: Sort,

    // Filtering options
    /// Exclude words containing fewer than min chars.
    #[arg(short, long, value_name = "COUNT")]
    min_chars: Option<usize>,

    /// Exclude words appearing fewer than min times.
    #[arg(short = 'M', long, value_name = "COUNT")]
    min_count: Option<usize>,

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

    // Performance options
    /// Use parallel processing for word counting.
    #[arg(short = 'p', long)]
    parallel: bool,
}

impl Args {
    pub const fn get_input(&self) -> &String {
        &self.input
    }

    pub const fn get_output(&self) -> &Option<PathBuf> {
        &self.output
    }

    pub const fn is_verbose(&self) -> bool {
        self.verbose
    }

    const fn get_delimiter(&self) -> &String {
        &self.delimiter
    }

    pub fn get_unescaped_delimiter(&self) -> Result<String> {
        unescape(self.get_delimiter().as_str()).with_context(|| "Failed to unescape delimiter")
    }

    const fn get_concurrency(&self) -> Concurrency {
        if self.parallel {
            Concurrency::Parallel
        } else {
            Concurrency::Sequential
        }
    }

    pub const fn get_formatting(&self) -> Formatting {
        Formatting::new(self.case, self.sort, self.format)
    }

    pub fn get_filters(&self) -> Result<Filters> {
        Filters::new(
            &self.min_chars,
            &self.min_count,
            self.exclude_words.as_ref(),
            self.exclude.as_ref(),
            self.include.as_ref(),
        )
        .with_context(|| "Failed to compile filter patterns")
    }

    pub fn get_performance(&self, size_hint: SizeHint) -> Performance {
        Performance::default()
            .with_concurrency(self.get_concurrency())
            .with_size_hint(size_hint)
    }

    pub fn get_options(&self, size_hint: SizeHint) -> Result<Options> {
        let formatting = self.get_formatting();
        let filters = self.get_filters()?;
        let performance = self.get_performance(size_hint);

        Ok(Options::new(formatting, filters, performance))
    }
}
