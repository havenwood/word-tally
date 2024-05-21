//! A tally of words with a count of times each appears
//!
//! A `WordTally` represents a tally of the total number of times each word
//! appears in an input source that implements `Read`. When a `WordTally` is
//! constructed, the provided input is iterated over line by line to count words.
//! Ordered pairs of words and their count are stored in the `tally` field.
//!
//! # `Case` and `Sort` enum options
//!
//! In addition to source input, a `WordTally` is contstructed with options for
//! `Case` normalization and `Sort` order. `Case` options include `Original`
//! (case sensitive) and `Lower` or `Upper` case normalization. `Sort` order can
//! be `Unsorted` or sorted `Desc` (descending) or `Asc` (ascending). A `tally`
//! can be sorted upon contruction or sorted later with the `sort` method.
//! Sorting doesn't impact the other `count`, `uniq_count` or `avg` fields.
//!
//! # Examples
//!
//! ```
//! use word_tally::{Case, Sort, WordTally};
//!
//! let input = "Cinquedea".as_bytes();
//! let words = WordTally::new(input, Case::Lower, Sort::Desc);
//! let expected_tally = vec![("cinquedea".to_string(), 1)];
//!
//! assert_eq!(words.tally(), expected_tally);
//! ```

use clap::ValueEnum;
use core::cmp::Reverse;
use core::hash::{Hash, Hasher};
use std::collections::HashMap;
use std::io::{self, BufRead, BufReader, Lines, Read};
use unicode_segmentation::UnicodeSegmentation;

/// A `WordTally` represents an ordered tally of words paired with their count.
#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
pub struct WordTally {
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

/// Since the other fields are derived from it, hash by just the `tally`.
impl Hash for WordTally {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.tally.hash(state);
    }
}

/// The `tally` field `Vec` is the best way to get at the ordered tally.
impl From<WordTally> for Vec<(String, u64)> {
    fn from(word_tally: WordTally) -> Self {
        word_tally.tally
    }
}

/// Word case normalization.
#[derive(Clone, Copy, Debug, Default, ValueEnum)]
pub enum Case {
    Original,
    Upper,
    #[default]
    Lower,
}

/// Sort order by count.
#[derive(Clone, Copy, Debug, Default, ValueEnum)]
pub enum Sort {
    #[default]
    Desc,
    Asc,
    Unsorted,
}

/// `WordTally` fields are eagerly populated upon construction and exposed by getter methods.
impl WordTally {
    /// Constructs a new `WordTally` from a source that implements `Read` like file or stdin.
    pub fn new<T: Read>(input: T, case: Case, order: Sort) -> Self {
        let lines = Self::lines(input);
        let tally_map = Self::tally_map(lines, case);
        let count = tally_map.values().sum();
        let tally = Vec::from_iter(tally_map);
        let uniq_count = tally.len();
        let avg = Self::calculate_avg(count, uniq_count);
        let mut word_tally = Self {
            tally,
            count,
            uniq_count,
            avg,
        };

        word_tally.sort(order);

        word_tally
    }

    /// Sorts the `tally` field in place if a sort order other than `Unsorted` is provided.
    pub fn sort(&mut self, order: Sort) {
        match order {
            Sort::Desc => self
                .tally
                .sort_unstable_by_key(|&(_, count)| Reverse(count)),
            Sort::Asc => self.tally.sort_unstable_by_key(|&(_, count)| count),
            Sort::Unsorted => (),
        }
    }

    /// Gets the `tally` field.
    pub fn tally(self) -> Vec<(String, u64)> {
        self.tally
    }

    /// Gets the `uniq_count` field.
    pub const fn uniq_count(&self) -> usize {
        self.uniq_count
    }

    /// Gets the `count` field.
    pub const fn count(&self) -> u64 {
        self.count
    }

    /// Gets the `avg` field.
    pub const fn avg(&self) -> Option<f64> {
        self.avg
    }

    /// Calculates an approximate mean average word count if there are words.
    /// Note: Casting `u64` to `f64` and floating point arithmatic cause a loss of precision.
    fn calculate_avg(count: u64, uniq_count: usize) -> Option<f64> {
        (count > 0).then(|| count as f64 / uniq_count as f64)
    }

    /// Creates a line buffer reader result from the input source.
    fn lines(input: impl Read) -> Lines<BufReader<impl Read>> {
        io::BufReader::new(input).lines()
    }

    /// Creates a tally of optionally normalized words from a line buffer reader.
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
