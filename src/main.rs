//! `word-tally` tallies and outputs the count of words from a given input.

pub(crate) mod args;
pub(crate) mod verbose;

use std::{
    fs, io,
    io::{Read, Write},
    process::ExitCode,
};

use anyhow::Result;
use clap::Parser;
use rayon::prelude::*;
use word_tally::{Io, Output, Reader, TallyMap, View, WordTally, WordTallyError};

use crate::{args::Args, verbose::Verbose};

type SourceProcessor = fn(&str, &word_tally::Options) -> Result<TallyMap>;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            let mut stderr = Output::stderr();
            stderr.write_all(format!("Error: {err}\n").as_bytes()).ok();
            ExitCode::from(u8::from(word_tally::exit_code::ExitCode::from(&err)))
        }
    }
}

fn run() -> Result<()> {
    let args = Args::parse();
    let sources = args.sources();
    let options = args.to_options()?;

    options.init_thread_pool_if_parallel()?;

    let tally_map = tally_sources(sources, &options)?;
    let word_tally = WordTally::from_tally_map(tally_map, &options);

    if args.verbose() {
        let mut verbose = Verbose::default();
        verbose.write_info(&word_tally, &sources.join(", "))?;
    }

    let mut output = Output::try_from(args.output())?;
    output.write_formatted_tally(&word_tally)?;

    Ok(())
}

// I/O processings orchestration

fn tally_sources(sources: &[String], options: &word_tally::Options) -> Result<TallyMap> {
    match options.io() {
        Io::Stream => tally_sequential(sources, options, process_with_reader),
        Io::ParallelStream => tally_parallel(sources, options, process_with_reader),
        Io::ParallelMmap => tally_parallel(sources, options, process_with_mmap),
        Io::ParallelInMemory => tally_parallel(sources, options, process_with_bytes),
        Io::ParallelBytes => Err(WordTallyError::BytesWithPath.into()),
    }
}

fn tally_sequential(
    sources: &[String],
    options: &word_tally::Options,
    processor: SourceProcessor,
) -> Result<TallyMap> {
    sources
        .iter()
        .map(|source| processor(source, options))
        .try_fold(TallyMap::new(), |acc, tally| tally.map(|t| acc.merge(t)))
}

fn tally_parallel(
    sources: &[String],
    options: &word_tally::Options,
    processor: SourceProcessor,
) -> Result<TallyMap> {
    sources
        .par_iter()
        .map(|source| processor(source, options))
        .try_reduce(TallyMap::new, |acc, tally| Ok(acc.merge(tally)))
}

// Processing based on I/O mode

fn process_with_reader(source: &str, options: &word_tally::Options) -> Result<TallyMap> {
    let reader = Reader::try_from(source)?;
    TallyMap::from_reader(&reader, options)
}

fn process_with_mmap(source: &str, options: &word_tally::Options) -> Result<TallyMap> {
    let view = View::try_from(source)?;
    TallyMap::from_view(&view, options)
}

fn process_with_bytes(source: &str, options: &word_tally::Options) -> Result<TallyMap> {
    let bytes = if source == "-" {
        let mut buffer = Vec::new();
        io::stdin().lock().read_to_end(&mut buffer)?;
        buffer
    } else {
        fs::read(source)?
    };
    let view = View::from(bytes);
    TallyMap::from_view(&view, options)
}
