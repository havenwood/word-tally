#![feature(unix_sigpipe)]
#![warn(clippy::nursery, clippy::pedantic)]
#![warn(deprecated_in_future, future_incompatible)]
#![warn(let_underscore)]
#![warn(nonstandard_style)]
#![warn(rust_2018_compatibility)]
#![warn(rust_2018_idioms)]
#![warn(rust_2021_compatibility)]
#![warn(rust_2024_compatibility)]
#![warn(unused)]

use clap::Parser;
use clap_stdin::FileOrStdin;
use core::cmp::Reverse;
use core::panic;
use std::collections::HashMap;
use std::io::{self, BufRead, BufReader, Lines, Read};
use std::path::PathBuf;
use unescaper::unescape;
use unicode_segmentation::UnicodeSegmentation;

#[unix_sigpipe = "sig_dfl"]
fn main() -> Result<(), unescaper::Error> {
    let args = Args::parse();
    let word_tally = WordTally::new(&args.input, !args.no_sort);
    let delimiter = unescape(&args.delimiter)?;

    if args.verbose {
        println!("source{delimiter}{:#?}", args.input.source);
    }

    if args.debug {
        println!("delimiter{delimiter}{delimiter:#?}");
        println!("sorted{delimiter}{}", word_tally.sorted);
        println!("verbose{delimiter}{}", args.verbose);
        println!("debug{delimiter}{}", args.debug);
    }

    if args.verbose || args.debug {
        println!();
    }

    for (word, count) in word_tally.tally {
        println!("{word}{delimiter}{count}");
    }

    Ok(())
}

#[derive(Debug, Parser)]
#[command(about, version)]
struct Args {
    #[clap(default_value = "-")]
    input: FileOrStdin<PathBuf>,
    #[clap(
        short = 'D',
        long,
        default_value = ": ",
        help = "Delimiter between word and count"
    )]
    delimiter: String,
    #[arg(short, long, help = "Unsorted word count order")]
    no_sort: bool,
    #[arg(short, long, help = "Additional debugging information")]
    debug: bool,
    #[arg(short, long, help = "Verbose command details")]
    verbose: bool,
}

#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct WordTally {
    pub sorted: bool,
    pub tally: Vec<(String, u64)>,
}

impl WordTally {
    #[must_use]
    pub fn new(input: &FileOrStdin<PathBuf>, sort: bool) -> Self {
        let mut word_tally = Self {
            sorted: false,
            tally: Vec::from_iter(Self::tally(Self::lines(input))),
        };

        if sort {
            word_tally.sort();
        }

        word_tally
    }

    pub fn sort(&mut self) {
        if self.sorted {
            return;
        }

        self.tally
            .sort_unstable_by_key(|&(_, count)| Reverse(count));
        self.sorted = true;
    }

    fn lines(input: &FileOrStdin<PathBuf>) -> Lines<BufReader<impl Read>> {
        match input.into_reader() {
            Ok(readable) => io::BufReader::new(readable).lines(),
            Err(err) => panic!("{err} -- {:#?}", input.source),
        }
    }

    fn tally(lines: io::Lines<BufReader<impl Read>>) -> HashMap<String, u64> {
        let mut tally = HashMap::new();

        for line in lines.map_while(Result::ok) {
            line.unicode_words().for_each(|word| {
                tally
                    .entry(word.to_lowercase())
                    .and_modify(|count| *count += 1)
                    .or_insert(1);
            });
        }

        tally
    }
}
