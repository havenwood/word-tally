#![feature(f128)]

use anyhow::{Context, Result};
use clap::ValueEnum;
use clap_stdin::FileOrStdin;
use core::cmp::Reverse;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::io::{self, BufRead, BufReader, Lines, Read};
use std::path::PathBuf;
use unicode_segmentation::UnicodeSegmentation;

/// A `WordTally` represents an ordered tally of words paired with their count.
#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
pub struct WordTally {
    /// Whether the `tally` field has been sorted by the `sort` method.
    sorted: bool,
    /// Ordered pairs of words and the count of times they appear.
    tally: Vec<(String, u64)>,
    /// The sum of all words tallied.
    count: u64,
    /// The sum of uniq words tallied.
    uniq_count: usize,
    /// The mean average count per word, if there are words.
    avg: Option<f64>,
}

impl Eq for WordTally {}

impl Hash for WordTally {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.tally.hash(state);
    }
}

impl From<WordTally> for Vec<(String, u64)> {
    fn from(word_tally: WordTally) -> Self {
        word_tally.tally
    }
}

#[derive(Clone, Copy, Debug, Default, ValueEnum)]
pub enum Case {
    Original,
    Upper,
    #[default]
    Lower,
}

#[derive(Clone, Copy, Debug, Default, ValueEnum)]
pub enum Sort {
    #[default]
    Desc,
    Asc,
    Unsorted,
}

impl WordTally {
    /// Constructs a new `WordTally` from a file or stdin source input.
    pub fn new(input: &FileOrStdin<PathBuf>, case: Case, order: Sort) -> Result<Self> {
        let lines = Self::lines(input)?;
        let tally_map = Self::tally_map(lines, case);
        let count = tally_map.values().sum();
        let tally = Vec::from_iter(tally_map);
        let uniq_count = tally.len();
        let avg = Self::calculate_avg(count, uniq_count);
        let mut word_tally = Self {
            sorted: false,
            tally,
            count,
            uniq_count,
            avg,
        };

        word_tally.sort(order);

        Ok(word_tally)
    }

    /// Sorts the `tally` field in place or does nothing if already sorted.
    pub fn sort(&mut self, order: Sort) {
        match order {
            Sort::Desc => self
                .tally
                .sort_unstable_by_key(|&(_, count)| Reverse(count)),
            Sort::Asc => self.tally.sort_unstable_by_key(|&(_, count)| count),
            Sort::Unsorted => return,
        }

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

    /// Gets the `uniq_count` field.
    pub fn uniq_count(&self) -> usize {
        self.uniq_count
    }

    /// Gets the `count` field.
    pub fn count(&self) -> u64 {
        self.count
    }

    /// Gets the `avg` field.
    pub fn avg(&self) -> Option<f64> {
        self.avg
    }

    /// Calculates the mean average word count if there are words.
    pub fn calculate_avg(count: u64, uniq_count: usize) -> Option<f64> {
        (count > 0).then(|| (count as f128 / uniq_count as f128) as f64)
    }

    /// Creates a line buffer reader from a file or stdin source.
    fn lines(input: &FileOrStdin<PathBuf>) -> Result<Lines<BufReader<impl Read>>> {
        let reader = input
            .into_reader()
            .with_context(|| format!("Failed to read {:#?}", input.source))?;

        Ok(io::BufReader::new(reader).lines())
    }

    /// Creates a tally of words from a line buffer reader.
    fn tally_map(lines: io::Lines<BufReader<impl Read>>, case: Case) -> HashMap<String, u64> {
        let mut tally = HashMap::new();

        for line in lines.map_while(Result::ok) {
            line.unicode_words().for_each(|unicode_word| {
                let word = match case {
                    Case::Lower => unicode_word.to_lowercase(),
                    Case::Upper => unicode_word.to_uppercase(),
                    Case::Original => unicode_word.to_owned(),
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
