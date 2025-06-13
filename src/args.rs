//! Command-line argument parsing and access.

use std::{fmt::Display, path::PathBuf};

use clap::{ArgAction, Parser};
use word_tally::{
    Count, Options, WordTallyError,
    options::{
        case::Case, encoding::Encoding, filters::Filters, io::Io, performance::Performance,
        serialization::Serialization, sort::Sort,
    },
};

/// A utility for tallying word frequencies in text.
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Parser)]
#[command(
    name = "word-tally",
    author,
    version,
    about,
    long_about = "Tally word frequencies with customizable options for sorting, filtering, and output formatting"
)]
pub(crate) struct Args {
    /// File paths to use as input (use "-" for stdin).
    #[arg(value_name = "PATHS", default_value = "-")]
    sources: Vec<String>,

    // Performance options
    /// I/O strategy.
    #[arg(short = 'I', long, value_enum, default_value_t = Io::ParallelStream, value_name = "STRATEGY")]
    io: Io,

    /// Text encoding mode for validation and word boundary detection.
    #[arg(short = 'e', long, value_enum, default_value_t = Encoding::Unicode, value_name = "ENCODING")]
    encoding: Encoding,

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
    #[arg(short = 'n', long, value_name = "COUNT")]
    min_count: Option<Count>,

    /// Exclude words from a comma-delimited list.
    #[arg(short = 'w', long, use_value_delimiter = true, value_name = "WORDS")]
    exclude_words: Option<Vec<String>>,

    /// Include only words matching a regex pattern.
    #[arg(short = 'i', long, value_name = "PATTERNS", action = ArgAction::Append)]
    include: Option<Vec<String>>,

    /// Exclude words matching a regex pattern.
    #[arg(short = 'x', long, value_name = "PATTERNS", action = ArgAction::Append)]
    exclude: Option<Vec<String>>,

    // Output options
    /// Output format.
    #[arg(short = 'f', long, default_value = "text", value_name = "FORMAT", value_parser = ["text", "json", "csv"])]
    format: String,

    /// Delimiter between field and value.
    #[arg(
        short = 'd',
        long = "field-delimiter",
        default_value = " ",
        value_name = "VALUE"
    )]
    field_delimiter: String,

    /// Delimiter between entries.
    #[arg(short = 'D', long, default_value = "\n", value_name = "VALUE")]
    entry_delimiter: String,

    /// Write output to file rather than stdout.
    #[arg(short, long, value_name = "PATH")]
    output: Option<PathBuf>,

    /// Print verbose details.
    #[arg(short = 'v', long)]
    verbose: bool,
}

impl Args {
    /// Get the input file paths.
    pub(crate) fn sources(&self) -> &[String] {
        &self.sources
    }

    /// Get the output file path.
    pub(crate) const fn output(&self) -> Option<&PathBuf> {
        self.output.as_ref()
    }

    /// Get the verbose flag.
    pub(crate) const fn verbose(&self) -> bool {
        self.verbose
    }

    /// Parse command-line arguments and convert them to word-tally `Options`.
    pub(crate) fn to_options(&self) -> Result<Options, WordTallyError> {
        Options::try_from(self)
    }

    /// Helper to create filters from arguments.
    fn build_filters(&self) -> Result<Filters, WordTallyError> {
        self.build_filters_impl()
    }

    /// Implementation of `build_filters` that works for both borrowed and owned.
    fn build_filters_impl(&self) -> Result<Filters, WordTallyError> {
        Ok(Filters::default())
            .map(|f| match self.min_chars {
                Some(min) => f.with_min_chars(min),
                None => f,
            })
            .map(|f| match self.min_count {
                Some(min) => f.with_min_count(min),
                None => f,
            })
            .map(|f| match &self.exclude_words {
                Some(words) => f.with_exclude_words(words.clone()),
                None => f,
            })
            .and_then(|f| match &self.exclude {
                Some(patterns) => f
                    .with_exclude_patterns(patterns)
                    .map_err(|e| Self::pattern_error("exclude", &e)),
                None => Ok(f),
            })
            .and_then(|f| match &self.include {
                Some(patterns) => f
                    .with_include_patterns(patterns)
                    .map_err(|e| Self::pattern_error("include", &e)),
                None => Ok(f),
            })
    }

    /// Convert a pattern compilation error into a `WordTallyError`.
    fn pattern_error(kind: &str, e: &impl Display) -> WordTallyError {
        WordTallyError::Pattern {
            kind: kind.to_string(),
            message: e.to_string(),
        }
    }
}

/// Converts command-line arguments to `Options`.
impl TryFrom<&Args> for Options {
    type Error = WordTallyError;

    fn try_from(args: &Args) -> Result<Self, Self::Error> {
        let serialization = match args.format.as_str() {
            "text" => Serialization::text()
                .with_field_delimiter(&args.field_delimiter)
                .with_entry_delimiter(&args.entry_delimiter),
            "json" => Serialization::Json,
            "csv" => Serialization::Csv,
            _ => unreachable!("clap should validate format values"),
        };

        Ok(Self::new(
            args.case,
            args.sort,
            serialization,
            args.build_filters()?,
            args.io,
            Performance::from_env(),
        )
        .with_encoding(args.encoding))
    }
}
