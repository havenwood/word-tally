//! `word-tally` tallies and outputs the count of words from a given input.

pub(crate) mod args;
pub(crate) mod input;
pub(crate) mod output;
pub(crate) mod verbose;

use anyhow::Result;
use args::Args;
use clap::Parser;
use input::Input;
use output::Output;
use unescaper::unescape;
use verbose::Verbose;
use word_tally::{Filters, Options, WordTally};

fn main() -> Result<()> {
    let args = Args::parse();
    let delimiter = unescape(&args.delimiter)?;
    let input = Input::from_args(&args.input)?;
    let source = input.source();
    let mut output = Output::from_args(&args.output)?;

    let reader = input.get_reader(&source)?;
    let options = Options::new(args.case, args.sort);
    let filters = Filters::new(&args.min_chars, &args.min_count, args.exclude);

    let word_tally = WordTally::new(reader, options, filters);

    if args.verbose {
        let mut verbose = Verbose::new(Output::stderr(), &word_tally, &delimiter, &source);
        verbose.log()?;
    }

    for (word, count) in word_tally.tally() {
        output.write_line(&format!("{word}{delimiter}{count}\n"))?;
    }
    output.flush()?;

    Ok(())
}
