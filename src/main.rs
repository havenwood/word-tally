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
use word_tally::{TallyMap, WordTally};

fn main() {
    match run() {
        Ok(()) => process::exit(exit_code::SUCCESS),
        Err(err) => {
            eprintln!("Error: {err}");
            process::exit(exit_code::from_error(&err));
        }
    }
}

fn run() -> Result<()> {
    // Parse arguments and prepare options
    let args = Args::parse();
    let sources = args.get_sources();
    let options = args.get_options()?;

    // Initialize processing mode (thread pool for parallel)
    options.processing().initialize(options.performance())?;

    // Process inputs and aggregate results
    let inputs = sources
        .iter()
        .map(|source| Input::new(source, options.io()))
        .collect::<Result<Vec<_>>>()?;

    let tally_map = inputs
        .iter()
        .map(|input| TallyMap::from_input(input, &options))
        .try_fold(TallyMap::new(), |acc, result| {
            result.map(|tally| acc.merge(tally))
        })?;

    // Create a `WordTally` from the merged `TallyMap`
    let word_tally = WordTally::from_tally_map(tally_map, &options);

    // Optional verbose output
    if args.is_verbose() {
        let paths: Vec<_> = inputs.iter().map(word_tally::Input::source).collect();
        let mut verbose = Verbose::default();
        verbose.write_verbose_info(&word_tally, &paths.join(", "))?;
    }

    // Primary output
    let output_option = args.get_output().map(ToOwned::to_owned);
    let mut output = Output::new(&output_option)?;
    output.write_formatted_tally(&word_tally)?;

    Ok(())
}
