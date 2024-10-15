use clap::Parser;
use std::path::PathBuf;
use word_tally::{Case, Sort};

#[derive(Debug, Parser)]
#[command(about, version)]
pub struct Args {
    /// File path to use as input rather than stdin ("-").
    #[arg(value_name = "PATH", default_value = "-", value_parser = clap::value_parser!(PathBuf))]
    pub input: PathBuf,

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
    #[arg(short, long, use_value_delimiter = true, value_name = "WORDS")]
    pub exclude: Option<Vec<String>>,

    /// Delimiter between keys and values.
    #[arg(short = 'D', long, default_value = " ", value_name = "VALUE")]
    pub delimiter: String,

    /// Write output to file rather than stdout.
    #[arg(short, long, value_name = "PATH")]
    pub output: Option<PathBuf>,

    /// Print verbose details.
    #[arg(short, long)]
    pub verbose: bool,

    /// Print debugging information.
    #[arg(short, long)]
    pub debug: bool,
}
