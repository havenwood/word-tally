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
use core::ops::Not;
use std::fs::File;
use std::io::{LineWriter, Write};
use unescaper::unescape;
use word_tally::WordTally;

fn main() -> Result<()> {
    let args = Args::parse();
    let word_tally = WordTally::new(&args.input, args.case_sensitive, args.no_sort.not());
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
        eprintln!("case sensitive{delimiter}{}", args.case_sensitive);
        eprintln!("sorted{delimiter}{}", word_tally.sorted());
        eprintln!("verbose{delimiter}{}", args.verbose);
        eprintln!("debug{delimiter}{}", args.debug);
    }

    if (args.verbose || args.debug) && word_tally.tally().is_empty().not() {
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
