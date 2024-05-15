#![warn(clippy::nursery)]
#![warn(future_incompatible)]
#![warn(let_underscore)]
#![warn(nonstandard_style)]
#![warn(rust_2018_compatibility)]
#![warn(rust_2018_idioms)]
#![warn(rust_2021_compatibility)]
#![warn(rust_2024_compatibility)]
#![warn(unused)]

pub(crate) mod args;

use crate::args::Args;

use anyhow::Result;
use clap::Parser;
use std::fs::File;
use std::io::{LineWriter, Write};
use unescaper::unescape;
use word_tally::*;

fn main() -> Result<()> {
    let args = Args::parse();
    let word_tally = WordTally::new(&args.input, args.case, args.sort)?;
    let delimiter = unescape(&args.delimiter)?;

    if args.verbose {
        eprintln!("source{delimiter}{:#?}", args.input.source);
        eprintln!("total words{delimiter}{}", word_tally.count());
        eprintln!("unique words{delimiter}{}", word_tally.uniq_count());

        if let Some(avg) = word_tally.avg() {
            eprintln!("average word count{delimiter}{avg:.3}");
        }
    }

    if args.debug {
        eprintln!("delimiter{delimiter}{delimiter:#?}");
        match args.case {
            Case::Sensitive => eprintln!("case sensitive{delimiter}true"),
            Case::Insensitive => eprintln!("case sensitive{delimiter}false"),
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

    match args.output {
        Some(path) => {
            let file = File::create(path)?;
            let mut writer = LineWriter::new(file);

            for (word, count) in word_tally.tally() {
                let line = format!("{word}{delimiter}{count}\n");
                writer.write_all(line.as_bytes())?;
            }

            writer.flush()?;
        }
        None => {
            for (word, count) in word_tally.tally() {
                println!("{word}{delimiter}{count}");
            }
        }
    };

    Ok(())
}
