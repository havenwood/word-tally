//! `word-tally` tallies and outputs the count of words from a given input.

pub(crate) mod args;
pub(crate) mod input;
pub(crate) mod output;
pub(crate) mod verbose;

use anyhow::{Context, Result};
use args::Args;
use clap::Parser;
use input::Input;
use output::Output;
use unescaper::unescape;
use verbose::Verbose;
use word_tally::{ExcludeWords, Filters, MinChars, MinCount, Options, WordTally};

fn main() -> Result<()> {
    let args = Args::parse();
    let delimiter = unescape(&args.delimiter)?;

    let input = Input::from_args(args.input)?;
    let input_file_name = input.file_name().unwrap_or("-").to_string();
    let reader = input
        .get_reader()
        .context(format!("Failed to read from {}.", input_file_name))?;

    let options = Options {
        case: args.case,
        sort: args.sort,
    };

    let filters = Filters {
        min_chars: args.min_chars.map(MinChars),
        min_count: args.min_count.map(MinCount),
        exclude: args.exclude.map(ExcludeWords),
    };

    let word_tally = WordTally::new(reader, options, filters);

    if args.verbose {
        let verbose = Verbose {};

        let mut stderr_output = Output::stderr();
        verbose.log(
            &mut stderr_output,
            &word_tally,
            &delimiter,
            &input_file_name,
        )?;
    };

    let mut output = Output::from_args(args.output)?;

    for (word, count) in word_tally.tally() {
        output.write_line(&format!("{word}{delimiter}{count}\n"))?;
    }

    output.flush()?;

    Ok(())
}
