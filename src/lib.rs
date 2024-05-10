#![feature(f128)]

use anyhow::Result;
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
    /// Whether the `tally` field has been sorted by the `sort` method.
    sorted: bool,
    /// Ordered pairs of words and the count of times they appear.
    tally: Vec<(String, u64)>,
}

impl WordTally {
    /// Constructs a new `WordTally` from a file or stdin source input.
    #[must_use]
    pub fn new(input: &FileOrStdin<PathBuf>, case_sensitive: bool, sort: bool) -> Self {
        let mut word_tally = Self {
            sorted: false,
            tally: Vec::from_iter(Self::tally_words(Self::lines(input), case_sensitive)),
        };

        if sort {
            word_tally.sort();
        }

        word_tally
    }

    /// Sorts the `tally` field in place or does nothing if already sorted.
    pub fn sort(&mut self) {
        if self.sorted {
            return;
        }

        self.tally
            .sort_unstable_by_key(|&(_, count)| Reverse(count));
        self.sorted = true;
    }

    /// Gets the `sorted` field.
    pub fn sorted(&self) -> bool {
        self.sorted
    }

    /// Gets the `tally` field.
    pub fn tally(&self) -> &Vec<(String, u64)> {
        &self.tally
    }

    /// Counts the sum of all unique words in the tally.
    pub fn uniq_count(&self) -> usize {
        self.tally.len()
    }

    /// Counts the total sum of all words in the tally.
    pub fn count(&self) -> u64 {
        self.tally.iter().map(|&(_, count)| count).sum()
    }

    /// Finds the mean average word count if there are words.
    pub fn avg(&self) -> Option<f64> {
        let count = self.count();

        (count > 0).then(|| (count as f128 / self.uniq_count() as f128) as f64)
    }

    /// Creates a line buffer reader from a file or stdin source.
    fn lines(input: &FileOrStdin<PathBuf>) -> Lines<BufReader<impl Read>> {
        match input.into_reader() {
            Ok(readable) => io::BufReader::new(readable).lines(),
            Err(err) => {
                eprintln!("{err} -- {:#?}", input.source);
                exit(1);
            }
        }
    }

    /// Creates a tally of words from a line buffer reader.
    fn tally_words(
        lines: io::Lines<BufReader<impl Read>>,
        case_sensitive: bool,
    ) -> HashMap<String, u64> {
        let mut tally = HashMap::new();

        for line in lines.map_while(Result::ok) {
            line.unicode_words().for_each(|unicode_word| {
                let word = if case_sensitive {
                    unicode_word.to_owned()
                } else {
                    unicode_word.to_lowercase()
                };

                tally
                    .entry(word)
                    .and_modify(|count| *count += 1)
                    .or_insert(1);
            });
        }

        tally
    }
}
