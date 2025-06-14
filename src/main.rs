//! `word-tally` tallies and outputs the count of words from a given input.

pub(crate) mod args;
pub(crate) mod verbose;

use std::{io::Write, process};

use anyhow::Result;
use clap::Parser;
use rayon::prelude::*;
use word_tally::{Io, Output, TallyMap, WordTally, WordTallyError};

use crate::{args::Args, verbose::Verbose};

fn main() -> process::ExitCode {
    match run() {
        Ok(()) => process::ExitCode::SUCCESS,
        Err(err) => {
            let mut stderr = Output::stderr();
            stderr.write_all(format!("Error: {err}\n").as_bytes()).ok();
            word_tally::exit_code::ExitCode::from(&err).into()
        }
    }
}

fn run() -> Result<()> {
    let args = Args::parse();
    let sources = args.sources();
    let options = args.to_options()?;

    options.init_thread_pool_if_parallel()?;

    let tally_map = match options.io() {
        Io::Stream => sources
            .iter()
            .map(|source| TallyMap::from_buffered_input(source, &options))
            .try_fold(TallyMap::new(), |acc, tally| tally.map(|t| acc.merge(t))),
        Io::ParallelStream => sources
            .par_iter()
            .map(|source| TallyMap::from_buffered_input(source, &options))
            .try_reduce(TallyMap::new, |acc, tally| Ok(acc.merge(tally))),
        Io::ParallelMmap => sources
            .par_iter()
            .map(|source| TallyMap::from_mapped_input(source, &options))
            .try_reduce(TallyMap::new, |acc, tally| Ok(acc.merge(tally))),
        Io::ParallelInMemory => sources
            .par_iter()
            .map(|source| TallyMap::from_bytes_input(source, &options))
            .try_reduce(TallyMap::new, |acc, tally| Ok(acc.merge(tally))),
        Io::ParallelBytes => Err(WordTallyError::PathInvalid.into()),
    }?;

    let word_tally = WordTally::from_tally_map(tally_map, &options);

    if args.verbose() {
        let mut verbose = Verbose::default();
        verbose.write_info(&word_tally, &sources.join(", "))?;
    }

    let mut output = Output::try_from(args.output())?;
    output.write_formatted_tally(&word_tally)?;

    Ok(())
}
