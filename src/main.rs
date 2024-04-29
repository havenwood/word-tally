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

#[unix_sigpipe = "sig_dfl"]
fn main() {
    let cli = Cli::parse();
    let word_tally = WordTally::new(&cli.path, !cli.no_sort);

    if cli.verbose {
        println!("path: {}", word_tally.path);
        println!("sorted: {}", word_tally.sorted);
    }

    if cli.debug {
        println!("verbose: {}", cli.verbose);
        println!("debug: {}", cli.debug);
    }

    if cli.verbose || cli.debug {
        println!();
    }

    for (word, count) in word_tally.tally {
        println!("{word}: {count}");
    }
}

use clap::Parser;

#[derive(Debug, Parser)]
#[command(version, about)]
struct Cli {
    path: String,
    #[arg(short, long, help = "Print additional details")]
    verbose: bool,
    #[arg(short, long, help = "Print unsorted word count")]
    no_sort: bool,
    #[arg(short, long, help = "Print debugging details")]
    debug: bool,
}

use core::cmp::Reverse;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead};
use std::path;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct WordTally {
    pub path: String,
    pub sorted: bool,
    pub tally: Vec<(String, u64)>,
}

impl WordTally {
    #[must_use]
    pub fn new(path: &str, sort: bool) -> Self {
        let mut word_tally = Self {
            path: Self::absolut_path(path),
            sorted: false,
            tally: Vec::from_iter(Self::tally(Self::lines(path))),
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
            .sort_unstable_by_key(|(_, count)| Reverse(*count));
        self.sorted = true;
    }

    fn absolut_path(path: &str) -> String {
        path::absolute(path)
            .expect("Absolute path not available")
            .into_os_string()
            .into_string()
            .expect("Absolute path encoding invalid")
    }

    fn lines(path: &str) -> io::Lines<io::BufReader<File>> {
        let file = File::open(path);
        assert!(file.is_ok(), "File not readable: {path}");

        io::BufReader::new(file.unwrap()).lines()
    }

    fn tally(lines: io::Lines<io::BufReader<File>>) -> HashMap<String, u64> {
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
