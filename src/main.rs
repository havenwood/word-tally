//! `word-tally` tallies and outputs the count of words from a given input.

pub(crate) mod args;
pub(crate) mod exit_code;
pub(crate) mod verbose;
pub(crate) use word_tally::input;
pub(crate) use word_tally::output;

use crate::exit_code::ExitCode;
use crate::input::Input;
use crate::output::Output;
use anyhow::Result;
use args::Args;
use clap::Parser;
use std::process;
use word_tally::WordTally;

fn main() {
    match run() {
        Ok(()) => process::exit(ExitCode::Success.into()),
        Err(err) => {
            eprintln!("Error: {}", err);
            process::exit(ExitCode::from_error(&err).into());
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
        verbose::handle_output(
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
