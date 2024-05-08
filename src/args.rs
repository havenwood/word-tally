use clap::Parser;
use clap_stdin::FileOrStdin;
use std::path::PathBuf;

/// Parses CLI arguments.
#[derive(Debug, Parser)]
#[command(about, version)]
pub struct Args {
    /// Source input for tallying words.
    #[clap(default_value = "-")]
    pub input: FileOrStdin<PathBuf>,
    /// Delimiter printed between keys and values. (Default is ": ", a colon and space.)
    #[clap(
        short = 'D',
        long,
        default_value = ": ",
        help = "Delimiter between word and count"
    )]
    pub delimiter: String,
    /// Use case sensitivity when tallying words. (Default is case insensitive.)
    #[arg(short, long, help = "Tally with case sensitivity")]
    pub case_sensitive: bool,
    /// Whether to start off unsorted. (Default is sorted descending.)
    #[arg(short, long, help = "Unsorted word count order")]
    pub no_sort: bool,
    /// Additional debugging information.
    #[arg(short, long, help = "Additional debugging information")]
    pub debug: bool,
    /// Additional file or stdin source input information.
    #[arg(short, long, help = "Verbose command details")]
    pub verbose: bool,
}
