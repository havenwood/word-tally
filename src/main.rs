//! `word-tally` tallies and outputs the count of words from a file or streamed input.

pub(crate) mod args;

use anyhow::{Context, Result};
use args::Args;
use clap::Parser;
use clap_stdin::Source;
use std::fs::File;
use std::io::{self, ErrorKind::BrokenPipe, LineWriter, StderrLock, Write};
use unescaper::unescape;
use word_tally::{Case, Sort, WordTally};

/// `Writer` is a boxed type for dynamic dispatch of the `Write` trait.
type Writer = Box<dyn Write>;

fn main() -> Result<()> {
    let args = Args::parse();
    let reader = args
        .input
        .into_reader()
        .with_context(|| format!("Failed to read {:#?}.", args.input.source))?;
    let word_tally = WordTally::new(reader, args.case, args.sort);
    let delimiter = unescape(&args.delimiter)?;

    if args.verbose || args.debug {
        log_details(&io::stderr(), &word_tally, &args, &delimiter)?;
    }

    write_tally(&io::stdout(), word_tally, &args, &delimiter)?;

    Ok(())
}

/// Log verbose and debug details to stderr.
fn log_details(
    stderr: &io::Stderr,
    word_tally: &WordTally,
    args: &Args,
    delimiter: &str,
) -> Result<()> {
    let mut stderr_lock = stderr.lock();

    if args.verbose {
        log_verbose(
            &mut stderr_lock,
            word_tally.count(),
            word_tally.uniq_count(),
            word_tally.avg(),
            &args.input.source,
            delimiter,
        )?;
    }

    if args.debug {
        log_debug(
            &mut stderr_lock,
            args.case,
            args.sort,
            args.verbose,
            args.debug,
            delimiter,
        )?;
    }

    if word_tally.count() > 0 {
        piping(stderr_lock.write_all(b"\n"))?;
    }

    piping(stderr_lock.flush())?;

    Ok(())
}

/// Log verbose details to stderr.
fn log_verbose(
    stderr: &mut StderrLock<'_>,
    count: u64,
    uniq_count: usize,
    maybe_avg: Option<f64>,
    source: &Source,
    delimiter: &str,
) -> Result<()> {
    let details = [
        format!("source{delimiter}{source:#?}\n"),
        format!("total-words{delimiter}{count}\n"),
        format!("unique-words{delimiter}{uniq_count}\n"),
    ];

    for detail in &details {
        piping(stderr.write_all(detail.as_bytes()))?;
    }

    if let Some(avg) = maybe_avg {
        piping(stderr.write_all(format!("average-word-count{delimiter}{avg:.3}\n").as_bytes()))?;
    }

    Ok(())
}

/// Log debug details to stderr.
fn log_debug(
    stderr: &mut StderrLock<'_>,
    case: Case,
    sort: Sort,
    verbose: bool,
    debug: bool,
    delimiter: &str,
) -> Result<()> {
    let case_name = match case {
        Case::Lower => "lower",
        Case::Upper => "upper",
        Case::Original => "original",
    };

    let sort_name = match sort {
        Sort::Asc => "asc",
        Sort::Desc => "desc",
        Sort::Unsorted => "unsorted",
    };

    let details = [
        format!("delimiter{delimiter}{delimiter:#?}\n"),
        format!("case{delimiter}{case_name}\n"),
        format!("order{delimiter}{sort_name}\n"),
        format!("verbose{delimiter}{verbose}\n"),
        format!("debug{delimiter}{debug}\n"),
    ];

    for detail in &details {
        piping(stderr.write_all(detail.as_bytes()))?;
    }

    Ok(())
}

/// Write word and count pairs to stdout, with a newline following each pair.
fn write_tally(
    stdout: &io::Stdout,
    word_tally: WordTally,
    args: &Args,
    delimiter: &str,
) -> Result<()> {
    let mut writer: Writer = match &args.output {
        Some(path) => Box::new(LineWriter::new(File::create(path)?)),
        None => Box::new(stdout.lock()),
    };

    for (word, count) in word_tally.tally() {
        let line = format!("{word}{delimiter}{count}\n");
        piping(writer.write_all(line.as_bytes()))?;
    }

    piping(writer.flush())?;

    Ok(())
}

/// Processes the result of a write operation, handling `BrokenPipe` errors
/// gracefully.
// This can be simplified once `-Zon-broken-pipe=kill` stabilizes and can be
// used to kill the program if it tries to write to a closed pipe.
fn piping(result: std::io::Result<()>) -> Result<()> {
    match result {
        Ok(_) => Ok(()),
        Err(err) => match err.kind() {
            BrokenPipe => Ok(()),
            _ => Err(err.into()),
        },
    }
}
