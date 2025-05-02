use clap::Parser;
use std::path::PathBuf;
use word_tally::{Case, Format, Sort};

#[derive(Debug, Parser)]
#[command(about, version)]
pub struct Args {
    /// File path to use as input rather than stdin ("-").
    #[arg(default_value = "-", value_name = "PATH")]
    pub input: String,

    /// Sort order.
    #[arg(short, long, default_value_t, value_enum, value_name = "ORDER")]
    pub sort: Sort,

    /// Case normalization.
    #[arg(short, long, default_value_t, value_enum, value_name = "FORMAT")]
    pub case: Case,

    /// Exclude words containing fewer than min chars.
    #[arg(short, long, value_name = "COUNT")]
    pub min_chars: Option<usize>,

    /// Exclude words appearing fewer than min times.
    #[arg(short = 'M', long, value_name = "COUNT")]
    pub min_count: Option<usize>,

    /// Exclude words from a comma-delimited list.
    #[arg(short = 'E', long, use_value_delimiter = true, value_name = "WORDS")]
    pub exclude_words: Option<Vec<String>>,

    /// Exclude words matching a regex pattern.
    #[arg(short = 'x', long, value_name = "PATTERN", action = clap::ArgAction::Append)]
    pub exclude: Option<Vec<String>>,

    /// Include only words matching a regex pattern.
    #[arg(short = 'i', long, value_name = "PATTERN", action = clap::ArgAction::Append)]
    pub include: Option<Vec<String>>,

    /// Delimiter between keys and values.
    #[arg(short, long, default_value = " ", value_name = "VALUE")]
    pub delimiter: String,

    /// Write output to file rather than stdout.
    #[arg(short, long, value_name = "PATH")]
    pub output: Option<PathBuf>,

    /// Print verbose details.
    #[arg(short = 'v', long)]
    pub verbose: bool,

    /// Output format.
    #[arg(short = 'f', long, default_value_t, value_enum, value_name = "FORMAT")]
    pub format: Format,

    /// Use parallel processing for word counting.
    #[arg(short = 'p', long)]
    pub parallel: bool,
}
