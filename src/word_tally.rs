use clap_stdin::FileOrStdin;
use core::cmp::Reverse;
use std::collections::HashMap;
use std::io::{self, BufRead, BufReader, Lines, Read};
use std::path::PathBuf;
use std::process::exit;
use unicode_segmentation::UnicodeSegmentation;

/// A `WordTally` represents an ordered tally of words paired with their count.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct WordTally {
    /// Whether the tally field has been sorted by the `sort` method.
    pub sorted: bool,
    /// Ordered pairs of words and the count of times they appear.
    pub tally: Vec<(String, u32)>,
}

impl WordTally {
    /// Constructs a new `WordTally` from a file or stdin source input.
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

    /// Sorts the tally field in place or does nothing if already sorted.
    pub fn sort(&mut self) {
        if self.sorted {
            return;
        }

        self.tally
            .sort_unstable_by_key(|&(_, count)| Reverse(count));
        self.sorted = true;
    }

    /// Creates a line buffer reader from a file or stdin source.
    fn lines(input: &FileOrStdin<PathBuf>) -> Lines<BufReader<impl Read>> {
        match input.into_reader() {
            Ok(readable) => io::BufReader::new(readable).lines(),
            Err(err) => {
                eprintln!("{err} -- {:#?}", input.source);
                exit(0);
            }
        }
    }

    /// Creates a tally of words from a line buffer reader.
    fn tally(lines: io::Lines<BufReader<impl Read>>) -> HashMap<String, u32> {
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
