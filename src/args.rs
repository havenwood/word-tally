use clap::Parser;
use clap_stdin::FileOrStdin;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(about, version)]
pub struct Args {
    /// Path to file to use as input rather than stdin ("-").
    #[clap(default_value = "-")]
    pub input: FileOrStdin<PathBuf>,

    /// Write output to specified file rather than stdout.
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Delimiter between keys and values.
    #[clap(short = 'D', long, default_value = ": ")]
    pub delimiter: String,

    /// Switch to tallying words with case sensitivity.
    #[arg(short, long)]
    pub case_sensitive: bool,

    /// Skip sorting by descending word count order.
    #[arg(short, long)]
    pub no_sort: bool,

    /// Print verbose source details.
    #[arg(short, long)]
    pub verbose: bool,

    /// Print debugging information.
    #[arg(short, long)]
    pub debug: bool,
}
