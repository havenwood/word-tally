//! `word-tally` tallies and outputs the count of words from a given input.

pub(crate) mod args;

use anyhow::{Context, Result};
use args::Args;
use clap::Parser;
use std::fs::File;
use std::io::{self, ErrorKind::BrokenPipe, LineWriter, Write};
use unescaper::unescape;
use word_tally::{Chars, Count, Filters, WordTally};

/// `Writer` is a boxed type for dynamic dispatch of the `Write` trait.
type Writer = Box<dyn Write>;

fn main() -> Result<()> {
    let args = Args::parse();
    let reader = args
        .input
        .into_reader()
        .with_context(|| format!("Failed to read {:#?}.", args.input.source))?;
    let filters = Filters {
        chars: Chars::min(args.min_chars),
        count: Count::min(args.min_count),
    };
    let word_tally = WordTally::new(reader, args.case, args.sort, filters);
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
    let mut w = Box::new(stderr.lock()) as Writer;

    if args.verbose {
        log_detail(
            &mut w,
            "source",
            delimiter,
            format!("{:?}", args.input.source),
        )?;
        log_detail(&mut w, "total-words", delimiter, word_tally.count())?;
        log_detail(&mut w, "unique-words", delimiter, word_tally.uniq_count())?;

        if let Some(avg) = word_tally.avg() {
            log_detail(&mut w, "average-word-count", delimiter, format!("{avg:.3}"))?;
        }
    }

    if args.debug {
        log_detail(&mut w, "delimiter", delimiter, format!("{delimiter:?}"))?;
        log_detail(&mut w, "case", delimiter, args.case)?;
        log_detail(&mut w, "order", delimiter, args.sort)?;
        log_detail(&mut w, "min-chars", delimiter, args.min_chars)?;
        log_detail(&mut w, "min-count", delimiter, args.min_count)?;
        log_detail(&mut w, "verbose", delimiter, args.verbose)?;
        log_detail(&mut w, "debug", delimiter, args.debug)?;
    }

    if word_tally.count() > 0 {
        log(&mut w, "\n")?;
    }

    piping(w.flush())?;

    Ok(())
}

/// Write word and count pairs to stdout, with a newline following each pair.
fn write_tally(
    stdout: &io::Stdout,
    word_tally: WordTally,
    args: &Args,
    delimiter: &str,
) -> Result<()> {
    let mut w: Writer = match &args.output {
        Some(path) => Box::new(LineWriter::new(File::create(path)?)),
        None => Box::new(stdout.lock()),
    };

    for (word, count) in word_tally.tally() {
        let line = format!("{word}{delimiter}{count}\n");
        log(&mut w, &line)?;
    }

    piping(w.flush())?;

    Ok(())
}

/// Log a formatted details line.
fn log_detail<T: std::fmt::Display>(
    w: &mut Writer,
    label: &str,
    delimiter: &str,
    value: T,
) -> Result<()> {
    let line = format!("{label}{delimiter}{value}\n");

    log(w, &line)
}

/// Log a line.
fn log(w: &mut Writer, line: &str) -> Result<()> {
    piping(w.write_all(line.as_bytes()))?;

    Ok(())
}

/// Processes the result of a write, handling `BrokenPipe` errors gracefully.
// This can be simplified once `-Zon-broken-pipe=kill` stabilizes and can be
// used to kill the program if it tries to write to a closed pipe.
fn piping(result: std::io::Result<()>) -> Result<()> {
    match result {
        Ok(()) => Ok(()),
        Err(err) => match err.kind() {
            BrokenPipe => Ok(()),
            _ => Err(err.into()),
        },
    }
}
