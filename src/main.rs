//! `word-tally` tallies and outputs the count of words from a given input.

pub(crate) mod args;
pub(crate) mod exit_code;
pub(crate) mod verbose;
pub(crate) use word_tally::input;
pub(crate) use word_tally::output;

use crate::input::Input;
use crate::output::Output;
use crate::verbose::Verbose;
use anyhow::Result;
use args::Args;
use clap::Parser;
use std::process;
use word_tally::WordTally;

fn main() {
    match run() {
        Ok(()) => process::exit(exit_code::SUCCESS),
        Err(err) => {
            eprintln!("Error: {}", err);
            process::exit(exit_code::from_error(&err));
        }
    }
}

fn run() -> Result<()> {
    // Parse arguments and prepare options and an input reader
    let args = Args::parse();
    let options = args.get_options()?;
    let input = Input::new(args.get_input(), options.io())?;

    // Construct a `WordTally` instance
    let word_tally = WordTally::new(&input, &options)?;

    // Optional verbose output
    if args.is_verbose() {
        let source = input.source();
        let mut verbose = Verbose::default();
        verbose.write_verbose_info(&word_tally, &source)?;
    }

    // Primary output
    let mut output = Output::new(args.get_output())?;
    output.write_formatted_tally(&word_tally)?;

    Ok(())
}
