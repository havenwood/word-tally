//! `word-tally` tallies and outputs the count of words from a given input.

pub(crate) mod args;
pub(crate) mod errors;
pub(crate) mod verbose;
pub(crate) use word_tally::input;
pub(crate) use word_tally::output;

use crate::errors::exit_code;
use crate::input::Input;
use crate::output::Output;
use anyhow::Result;
use args::Args;
use clap::Parser;
use word_tally::WordTally;

fn main() {
    match run() {
        Ok(()) => std::process::exit(exit_code::SUCCESS),
        Err(err) => {
            eprintln!("Error: {}", err);
            std::process::exit(exit_code(&err));
        }
    }
}

fn run() -> Result<()> {
    // Parse arguments and prepare an input reader
    let args = Args::parse();

    let options = args.get_options()?;
    let input = Input::new(args.get_input(), options.io())?;

    let source = input.source();
    let word_tally = WordTally::new(&input, &options)?;

    // Process output
    if args.is_verbose() {
        crate::verbose::handle_verbose_output(
            &word_tally,
            options.serialization().format(),
            options.serialization().delimiter(),
            &source,
        )?;
    }

    let mut output = Output::new(args.get_output())?;
    output.write_formatted_tally(
        word_tally.tally(),
        options.serialization().format(),
        options.serialization().delimiter(),
    )?;

    Ok(())
}
