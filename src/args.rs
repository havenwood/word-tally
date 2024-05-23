use clap::Parser;
use clap_stdin::FileOrStdin;
use std::path::PathBuf;
use word_tally::{Case, Sort};

#[derive(Debug, Parser)]
#[command(about, version)]
pub struct Args {
    /// File path to use as input rather than stdin ("-").
    #[arg(default_value = "-")]
    pub input: FileOrStdin<PathBuf>,

    /// Sort order.
    #[arg(short, long, default_value_t, value_enum, value_name = "ORDER")]
    pub sort: Sort,

    /// Case normalization.
    #[arg(short, long, default_value_t, value_enum, value_name = "FORMAT")]
    pub case: Case,

    /// Exclude words that contain fewer than min chars.
    #[arg(short, long, default_value_t = 1, value_name = "COUNT")]
    pub min_chars: usize,

    /// Exclude words that appear fewer than min times.
    #[arg(short = 'M', long, default_value_t = 1, value_name = "COUNT")]
    pub min_count: u64,

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
