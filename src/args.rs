//! Command-line argument parsing and access.

use anyhow::Result;
use clap::{ArgAction, Parser};
use std::path::PathBuf;

use word_tally::options::{
    case::Case,
    filters::Filters,
    io::Io,
    performance::Performance,
    serialization::{Format, Serialization},
    sort::Sort,
};
use word_tally::{Count, Options, WordTallyError};

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
    /// File paths to use as input (use "-" for stdin).
    #[arg(value_name = "PATHS", default_value = "-")]
    sources: Vec<String>,

    // Performance options
    /// I/O strategy.
    #[arg(short = 'I', long, value_enum, default_value_t = Io::Streamed, value_name = "STRATEGY")]
    io: Io,

    /// Disable parallel processing (use sequential).
    #[arg(short = 'S', long = "no-parallel", action = ArgAction::SetFalse)]
    parallel: bool,

    /// Enable parallel processing [default].
    #[arg(short = 'p', long = "parallel", overrides_with = "parallel")]
    _no_parallel: bool,

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
    #[arg(short = 'i', long, value_name = "PATTERNS", action = ArgAction::Append)]
    include: Option<Vec<String>>,

    /// Exclude words matching a regex pattern.
    #[arg(short = 'x', long, value_name = "PATTERNS", action = ArgAction::Append)]
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
    /// Get the input file paths.
    pub fn sources(&self) -> &[String] {
        &self.sources
    }

    /// Get the output file path.
    pub const fn output(&self) -> Option<&PathBuf> {
        self.output.as_ref()
    }

    /// Get the verbose flag.
    pub const fn verbose(&self) -> bool {
        self.verbose
    }

    /// Parse command-line arguments and convert them to word-tally `Options`.
    pub fn to_options(&self) -> Result<Options> {
        Ok(Options::new(
            self.case,
            self.sort,
            Serialization::new(self.format, &self.delimiter)?,
            self.build_filters()?,
            self.io,
            self.parallel.into(),
            Performance::from_env(),
        ))
    }

    /// Helper to create filters from arguments.
    fn build_filters(&self) -> Result<Filters> {
        Ok(Filters::default())
            .map(|f| match self.min_chars {
                Some(min) => f.with_min_chars(min),
                None => f,
            })
            .map(|f| match self.min_count {
                Some(min) => f.with_min_count(min),
                None => f,
            })
            .and_then(|f| match &self.exclude_words {
                Some(words) => f.with_unescaped_exclude_words(words),
                None => Ok(f),
            })
            .and_then(|f| match &self.exclude {
                Some(patterns) => f.with_exclude_patterns(patterns).map_err(|e| {
                    WordTallyError::Pattern {
                        kind: "exclude".to_string(),
                        message: e.to_string(),
                    }
                    .into()
                }),
                None => Ok(f),
            })
            .and_then(|f| match &self.include {
                Some(patterns) => f.with_include_patterns(patterns).map_err(|e| {
                    WordTallyError::Pattern {
                        kind: "include".to_string(),
                        message: e.to_string(),
                    }
                    .into()
                }),
                None => Ok(f),
            })
    }
}
