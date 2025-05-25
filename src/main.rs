//! `word-tally` tallies and outputs the count of words from a given input.

pub(crate) mod args;
pub(crate) mod verbose;
pub(crate) use word_tally::input;
pub(crate) use word_tally::output;

use crate::input::Input;
use crate::output::Output;
use crate::verbose::Verbose;
use anyhow::Result;
use args::Args;
use clap::Parser;
use rayon::prelude::*;
use std::process;
use word_tally::{Processing, TallyMap, WordTally, exit_code::ExitCode};

fn main() {
    match run() {
        Ok(()) => process::exit(ExitCode::Success.into()),
        Err(err) => {
            eprintln!("Error: {err}");
            process::exit(ExitCode::from_error(&err).into());
        }
    }
}

fn run() -> Result<()> {
    // Parse arguments and prepare options
    let args = Args::parse();
    let sources = args.sources();
    let options = args.to_options()?;

    // Process inputs and aggregate results
    let inputs = sources
        .iter()
        .map(|source| Input::new(source.as_str(), options.io()))
        .collect::<Result<Vec<_>>>()?;

    options.init_thread_pool_if_parallel()?;
    let tally_map = match options.processing() {
        Processing::Parallel => inputs
            .par_iter()
            .map(|input| TallyMap::from_input(input, &options))
            .try_reduce(TallyMap::new, |acc, tally| Ok(acc.merge(tally)))?,
        Processing::Sequential => inputs
            .iter()
            .map(|input| TallyMap::from_input(input, &options))
            .try_fold(TallyMap::new(), |acc, tally| tally.map(|t| acc.merge(t)))?,
    };

    // Create a `WordTally` from the merged `TallyMap`
    let word_tally = WordTally::from_tally_map(tally_map, &options);

    // Optional verbose output
    if args.verbose() {
        let paths: Vec<_> = inputs.iter().map(word_tally::Input::source).collect();
        let mut verbose = Verbose::default();
        verbose.write_verbose_info(&word_tally, &paths.join(", "))?;
    }

    // Primary output
    let mut output = Output::new(args.output().map(std::path::PathBuf::as_path))?;
    output.write_formatted_tally(&word_tally)?;

    Ok(())
}
