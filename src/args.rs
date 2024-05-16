use clap::Parser;
use clap_stdin::FileOrStdin;
use std::path::PathBuf;
use word_tally::{Case, Sort};

#[derive(Debug, Parser)]
#[command(about, version)]
pub struct Args {
    /// File path to use as input rather than stdin ("-").
    #[clap(default_value = "-")]
    pub input: FileOrStdin<PathBuf>,

    /// Order.
    #[arg(short, long, default_value_t, value_enum, value_name = "ORDER")]
    pub sort: Sort,

    /// Normalization.
    #[arg(short, long, default_value_t, value_enum, value_name = "FORMAT")]
    pub case: Case,

    /// Delimiter between keys and values.
    #[clap(short = 'D', long, default_value = " ", value_name = "VALUE")]
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
