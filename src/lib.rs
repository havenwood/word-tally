//! A tally of words with a count of the number of times each appears.
//!
//! A `WordTally` represents a tally of the total number of times each word
//! appears in an input source that implements `Read`. When a `WordTally` is
//! constructed, the provided input is iterated over line by line to count words.
//! Ordered pairs of words and their count are stored in the `tally` field.
//!
//! The `unicode-segmentation` Crate segments along "Word Bounaries" according
//! to the [Unicode Standard Annex #29](http://www.unicode.org/reports/tr29/).
//!
//! # `Case`, `Sort` and `Filters`
//!
//! In addition to source input, a `WordTally` is contstructed with options for
//! `Case` normalization, `Sort` order and word `Filters`. `Case` options include
//! `Original` (case sensitive) and `Lower` or `Upper` case normalization. `Sort`
//! order can be `Unsorted` or sorted `Desc` (descending) or `Asc` (ascending).
//! A `tally` can be sorted upon contruction or sorted later with the `sort` method.
//! Sorting doesn't impact the `count`, `uniq_count` or `avg` fields. `Filter`s can
//! be used to provide list of words that should or shouldn't be tallied.
//!
//! # Examples
//!
//! ```
//! use word_tally::{Case, Filters, Sort, WordTally};
//!
//! let input = "Cinquedea".as_bytes();
//! let words = WordTally::new(input, Case::Lower, Sort::Desc, Filters::default());
//! let expected_tally: Box<[(Box<str>, u64)]> = [("cinquedea".into(), 1)].into();
//!
//! assert_eq!(words.tally(), expected_tally);
//! ```
use clap::ValueEnum;
use core::cmp::Reverse;
use core::fmt::{self, Display, Formatter};
use core::hash::{Hash, Hasher};
use indexmap::IndexMap;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Read};
use unicode_segmentation::UnicodeSegmentation;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
pub struct WordTally {
    /// Ordered pairs of words and the count of times they appear.
    tally: Box<[(Box<str>, u64)]>,

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

/// A `tally` supports `iter` and can also be represented as a `Vec`.
impl From<WordTally> for Vec<(Box<str>, u64)> {
    fn from(word_tally: WordTally) -> Self {
        word_tally.tally.into_vec()
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

impl Display for Case {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
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

impl Display for Sort {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let order = match self {
            Self::Desc => "desc",
            Self::Asc => "asc",
            Self::Unsorted => "unsorted",
        };

        f.write_str(order)
    }
}

/// Filters for words to be included in the tally.
#[derive(Clone, Debug, Default, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct Filters {
    /// Word chars filters for tallying.
    pub min_chars: Option<MinChars>,

    /// Word count filters for tallying.
    pub min_count: Option<MinCount>,

    /// List of specific words to exclude for tallying.
    pub words_exclude: WordsExclude,

    /// List of specific words to only include for tallying.
    pub words_only: WordsOnly,
}

/// Min number of chars a word needs to be tallied.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct MinChars(pub usize);

impl Display for MinChars {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<usize> for MinChars {
    fn from(raw: usize) -> Self {
        Self(raw)
    }
}

/// Min count a word needs to be tallied.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct MinCount(pub u64);

impl Display for MinCount {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u64> for MinCount {
    fn from(raw: u64) -> Self {
        Self(raw)
    }
}

/// A list of words that should not be tallied.
#[derive(Clone, Debug, Default, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct WordsExclude(pub Option<Vec<String>>);

impl From<Vec<String>> for WordsExclude {
    fn from(raw: Vec<String>) -> Self {
        Self(Some(raw))
    }
}

/// A list of words that should only be tallied.
#[derive(Clone, Debug, Default, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct WordsOnly(pub Option<Vec<String>>);

impl From<Vec<String>> for WordsOnly {
    fn from(raw: Vec<String>) -> Self {
        Self(Some(raw))
    }
}

/// `WordTally` fields are eagerly populated upon construction and exposed by getter methods.
impl WordTally {
    /// Constructs a new `WordTally` from a source that implements `Read` like file or stdin.
    pub fn new<T: Read>(input: T, case: Case, order: Sort, filters: Filters) -> Self {
        let mut tally_map = Self::tally_map(input, case, filters.min_chars);
        Self::filter(&mut tally_map, filters, case);

        let count = tally_map.values().sum();
        let tally: Box<[_]> = tally_map.into_iter().collect();
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
    pub fn tally(self) -> Box<[(Box<str>, u64)]> {
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

    /// Creates a tally of normalized words from an input that implements `Read`.
    fn tally_map<T: Read>(
        input: T,
        case: Case,
        min_chars: Option<MinChars>,
    ) -> IndexMap<Box<str>, u64> {
        let mut tally = IndexMap::new();
        let lines = BufReader::new(input).lines();

        match min_chars {
            Some(MinChars(count)) => {
                for line in lines.map_while(Result::ok) {
                    line.unicode_words()
                        .filter(|word| word.graphemes(true).count() >= count)
                        .for_each(|word| {
                            *tally.entry(Self::normalize_case(word, case)).or_insert(0) += 1;
                        });
                }
            }
            None => {
                for line in lines.map_while(Result::ok) {
                    line.unicode_words().for_each(|word| {
                        *tally.entry(Self::normalize_case(word, case)).or_insert(0) += 1;
                    });
                }
            }
        }

        tally
    }

    /// Removes words from the `tally_map` based on any word `Filters`.
    fn filter(tally_map: &mut IndexMap<Box<str>, u64>, filters: Filters, case: Case) {
        // Remove any words that lack the minimum number of characters.
        if let Some(MinCount(min_count)) = filters.min_count {
            tally_map.retain(|_, &mut count| count >= min_count);
        }

        // Remove any words on the `exclude` word list.
        if let WordsExclude(Some(excludes)) = filters.words_exclude {
            let normalized_excludes: Vec<_> = excludes
                .iter()
                .map(|exclude| Self::normalize_case(exclude, case))
                .collect();
            tally_map.retain(|word, _| !normalized_excludes.contains(word));
        }

        // Remove any words absent from the `only` word list.
        if let WordsOnly(Some(exclusives)) = filters.words_only {
            let normalized_exclusives: Vec<_> = exclusives
                .iter()
                .map(|exclusive| Self::normalize_case(exclusive, case))
                .collect();
            tally_map.retain(|word, _| normalized_exclusives.contains(word));
        }
    }

    /// Normalizes word case if a `Case` other than `Case::Original` is provided.
    fn normalize_case(word: &str, case: Case) -> Box<str> {
        match case {
            Case::Lower => word.to_lowercase().into_boxed_str(),
            Case::Upper => word.to_uppercase().into_boxed_str(),
            Case::Original => Box::from(word),
        }
    }
}
