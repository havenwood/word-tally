//! `word-tally` tallies and outputs the count of words from a file or streamed input.

pub(crate) mod args;

use anyhow::{Context, Result};
use args::Args;
use clap::Parser;
use std::fs::File;
use std::io::{self, ErrorKind::BrokenPipe, LineWriter, Write};
use unescaper::unescape;
use word_tally::{Case, Sort, WordTally};

/// `Writer` is a boxed type for dynamic dispatch of the `Write` trait.
type Writer = Box<dyn Write>;

fn main() -> Result<()> {
    let args = Args::parse();
    let reader = args
        .input
        .into_reader()
        .with_context(|| format!("Failed to read {:#?}", args.input.source))?;
    let word_tally = WordTally::new(reader, args.case, args.sort);
    let delimiter = unescape(&args.delimiter)?;

    if args.verbose {
        eprintln!("source{delimiter}{:#?}", args.input.source);
        eprintln!("total-words{delimiter}{}", word_tally.count());
        eprintln!("unique-words{delimiter}{}", word_tally.uniq_count());

        if let Some(avg) = word_tally.avg() {
            eprintln!("average-word-count{delimiter}{avg:.3}");
        }
    }

    if args.debug {
        eprintln!("delimiter{delimiter}{delimiter:#?}");
        match args.case {
            Case::Lower => eprintln!("case{delimiter}lower"),
            Case::Upper => eprintln!("case{delimiter}upper"),
            Case::Original => eprintln!("case{delimiter}original"),
        }
        match args.sort {
            Sort::Asc => eprintln!("order{delimiter}asc"),
            Sort::Desc => eprintln!("order{delimiter}desc"),
            Sort::Unsorted => eprintln!("order{delimiter}unsorted"),
        }
        eprintln!("verbose{delimiter}{}", args.verbose);
        eprintln!("debug{delimiter}{}", args.debug);
    }

    if (args.verbose || args.debug) && word_tally.count() > 0 {
        eprintln!();
    }

    let mut writer: Writer = match args.output {
        Some(path) => Box::new(LineWriter::new(File::create(path)?)) as Writer,
        None => Box::new(io::stdout()) as Writer,
    };

    for (word, count) in word_tally.tally() {
        let line = format!("{word}{delimiter}{count}\n");

        // This can be simplified once `-Zon-broken-pipe=kill` stabilizes.
        // See: https://doc.rust-lang.org/nightly/unstable-book/compiler-flags/on-broken-pipe.html
        if let Err(err) = writer.write_all(line.as_bytes()) {
            return match err.kind() {
                BrokenPipe => Ok(()),
                _ => Err(err.into()),
            };
        }
    }

    writer.flush()?;

    Ok(())
}
