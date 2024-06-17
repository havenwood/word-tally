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
//! use word_tally::{Case, Filters, Sort, WordTally};
//!
//! let input = "Cinquedea".as_bytes();
//! let words = WordTally::new(input, Case::Lower, Sort::Desc, Filters::default());
//! let expected_tally = vec![("cinquedea".to_string(), 1)];
//!
//! assert_eq!(words.tally(), expected_tally);
//! ```
use clap::ValueEnum;
use core::cmp::Reverse;
use core::fmt;
use core::hash::{Hash, Hasher};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read};
use unicode_segmentation::UnicodeSegmentation;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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

impl fmt::Display for Case {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let case = match self {
            Self::Lower => "lower",
            Self::Upper => "upper",
            Self::Original => "original",
        };

        f.write_str(case)
    }
}

/// Sort order by count.
#[derive(Clone, Copy, Debug, Default, ValueEnum)]
pub enum Sort {
    #[default]
    Desc,
    Asc,
    Unsorted,
}

impl fmt::Display for Sort {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let order = match self {
            Self::Desc => "desc",
            Self::Asc => "asc",
            Self::Unsorted => "unsorted",
        };

        f.write_str(order)
    }
}

/// Filters for words to be included in the tally.
#[derive(Clone, Debug, Default, Eq, PartialEq, PartialOrd, Ord)]
pub struct Filters {
    /// Word chars filters for tallying.
    pub chars: Chars,

    /// Word count filters for tallying.
    pub count: Count,

    /// List of specific words to filter for tallying.
    pub words: Words,
}

/// Word chars filters for tallying.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, PartialOrd, Ord)]
pub struct Chars {
    /// Min number of chars in a word for it to be tallied.
    pub min: usize,
}

impl Chars {
    pub const fn min(size: usize) -> Self {
        Self { min: size }
    }
}

/// Word count filters for tallying.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, PartialOrd, Ord)]
pub struct Count {
    /// Min number of a word must occur to be tallied.
    pub min: u64,
}

impl Count {
    pub const fn min(size: u64) -> Self {
        Self { min: size }
    }
}

/// List of specific words to filter for tallying.
#[derive(Clone, Debug, Default, Eq, PartialEq, PartialOrd, Ord)]
pub struct Words {
    /// A list of words that should not be tallied.
    pub exclude: Option<Vec<String>>,

    /// A list of words to only tally.
    pub only: Option<Vec<String>>,
}

impl Words {
    pub const fn exclude(words: Option<Vec<String>>) -> Self {
        Self {
            exclude: words,
            only: None,
        }
    }

    pub const fn only(words: Option<Vec<String>>) -> Self {
        Self {
            only: words,
            exclude: None,
        }
    }
}

/// `WordTally` fields are eagerly populated upon construction and exposed by getter methods.
impl WordTally {
    /// Constructs a new `WordTally` from a source that implements `Read` like file or stdin.
    pub fn new<T: Read>(input: T, case: Case, order: Sort, filters: Filters) -> Self {
        let mut tally_map = Self::tally_map(input, case, filters.chars);
        if filters.count.min > 1 {
            tally_map.retain(|_, &mut count| count >= filters.count.min);
        }
        if let Some(excludes) = filters.words.exclude {
            let normalized_excludes: Vec<_> = excludes
                .iter()
                .map(|exclude| Self::normalize_case(exclude, case))
                .collect();
            tally_map.retain(|word, _| !normalized_excludes.contains(word));
        }
        if let Some(exclusives) = filters.words.only {
            let normalized_exclusives: Vec<_> = exclusives
                .iter()
                .map(|exclusive| Self::normalize_case(exclusive, case))
                .collect();
            tally_map.retain(|word, _| normalized_exclusives.contains(word));
        }
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

    /// Creates a tally of optionally normalized words from input that implements `Read`.
    fn tally_map<T: Read>(input: T, case: Case, chars: Chars) -> HashMap<String, u64> {
        let mut tally = HashMap::new();
        let lines = BufReader::new(input).lines();

        for line in lines.map_while(Result::ok) {
            line.unicode_words()
                .filter(|unicode_word| chars.min <= 1 || unicode_word.len() >= chars.min)
                .for_each(|unicode_word| {
                    let word = Self::normalize_case(unicode_word, case);

                    *tally.entry(word).or_insert(0) += 1;
                });
        }

        tally
    }

    /// Normalize case or use the original.
    fn normalize_case(word: &str, case: Case) -> String {
        match case {
            Case::Lower => word.to_lowercase(),
            Case::Upper => word.to_uppercase(),
            Case::Original => word.to_owned(),
        }
    }
}
