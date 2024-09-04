//! `word-tally` tallies and outputs the count of words from a given input.

pub(crate) mod args;

use anyhow::{Context, Result};
use args::Args;
use clap::Parser;
use std::fs::File;
use std::io::{self, ErrorKind::BrokenPipe, LineWriter, Write};
use std::path::PathBuf;
use unescaper::unescape;
use word_tally::{Case, Filters, MinChars, MinCount, Sort, WordTally, WordsExclude, WordsOnly};

/// `Writer` is a boxed type for dynamic dispatch of the `Write` trait.
type Writer = Box<dyn Write>;

/// `LogConfig` contains `Args` flags that may be used for logging.
struct LogConfig {
    verbose: bool,
    debug: bool,
    case: Case,
    sort: Sort,
    min_chars: usize,
    min_count: u64,
}

fn main() -> Result<()> {
    let Args {
        input,
        min_chars,
        min_count,
        exclude,
        only,
        case,
        sort,
        delimiter,
        verbose,
        debug,
        output,
    } = Args::parse();

    let source = input.filename().to_owned();

    let reader = input
        .into_reader()
        .with_context(|| format!("Failed to read {:?}.", source))?;

    let filters = Filters {
        min_chars: MinChars(min_chars),
        min_count: MinCount(min_count),
        words_exclude: WordsExclude(exclude),
        words_only: WordsOnly(only),
    };

    let word_tally = WordTally::new(reader, case, sort, filters);
    let delimiter = unescape(&delimiter)?;

    if verbose || debug {
        let log_config = LogConfig {
            verbose,
            debug,
            case,
            sort,
            min_chars,
            min_count,
        };

        log_details(&io::stderr(), &word_tally, &delimiter, &source, log_config)?;
    }

    write_tally(&io::stdout(), word_tally, output, &delimiter)?;

    Ok(())
}

/// Log verbose and debug details to stderr.
fn log_details(
    stderr: &io::Stderr,
    word_tally: &WordTally,
    delimiter: &str,
    source: &str,
    log_config: LogConfig,
) -> Result<()> {
    let mut w = Box::new(stderr.lock()) as Writer;

    if log_config.verbose {
        log_detail(&mut w, "source", delimiter, source)?;
        log_detail(&mut w, "total-words", delimiter, word_tally.count())?;
        log_detail(&mut w, "unique-words", delimiter, word_tally.uniq_count())?;

        if let Some(avg) = word_tally.avg() {
            log_detail(&mut w, "average-word-count", delimiter, format!("{avg:.3}"))?;
        }
    }

    if log_config.debug {
        log_detail(&mut w, "delimiter", delimiter, format!("{delimiter:?}"))?;
        log_detail(&mut w, "case", delimiter, log_config.case)?;
        log_detail(&mut w, "order", delimiter, log_config.sort)?;
        log_detail(&mut w, "min-chars", delimiter, log_config.min_chars)?;
        log_detail(&mut w, "min-count", delimiter, log_config.min_count)?;
        log_detail(&mut w, "verbose", delimiter, log_config.verbose)?;
        log_detail(&mut w, "debug", delimiter, log_config.debug)?;
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
    output: Option<PathBuf>,
    delimiter: &str,
) -> Result<()> {
    let mut w: Writer = match output {
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
