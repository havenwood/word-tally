#![warn(clippy::nursery, clippy::pedantic)]
#![warn(deprecated_in_future, future_incompatible)]
#![warn(let_underscore)]
#![warn(nonstandard_style)]
#![warn(rust_2018_compatibility)]
#![warn(rust_2018_idioms)]
#![warn(rust_2021_compatibility)]
#![warn(rust_2024_compatibility)]
#![warn(unused)]

pub(crate) mod args;
pub(crate) mod word_tally;

use crate::args::Args;
use crate::word_tally::WordTally;

use clap::Parser;
use unescaper::{unescape, Error};

fn main() -> Result<(), Error> {
    let args = Args::parse();
    let word_tally = WordTally::new(&args.input, !args.no_sort);
    let delimiter = unescape(&args.delimiter)?;

    if args.verbose {
        eprintln!("source{delimiter}{:#?}", args.input.source);
        eprintln!("unique words{delimiter}{}", word_tally.tally.len());
    }

    if args.debug {
        eprintln!("delimiter{delimiter}{delimiter:#?}");
        eprintln!("sorted{delimiter}{}", word_tally.sorted);
        eprintln!("verbose{delimiter}{}", args.verbose);
        eprintln!("debug{delimiter}{}", args.debug);
    }

    if args.verbose || args.debug {
        eprintln!();
    }

    for (word, count) in word_tally.tally {
        println!("{word}{delimiter}{count}");
    }

    Ok(())
}
