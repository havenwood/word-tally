use crate::Case;
use core::fmt::{self, Display, Formatter};
use indexmap::IndexMap;
use std::collections::HashSet;
use unicode_segmentation::UnicodeSegmentation;

use serde::{Deserialize, Serialize};

/// Filters for which words should be tallied.
#[derive(Clone, Debug, Default, Eq, PartialEq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Filters {
    /// Minimum characters required for a word.
    pub min_chars: Option<MinChars>,

    /// Minimum count for number of times a word must appear.
    pub min_count: Option<MinCount>,

    /// List of specific words to exclude.
    pub exclude: Option<ExcludeWords>,
}

impl Filters {
    /// Constructs `Filters`.
    pub fn new(
        min_chars: &Option<usize>,
        min_count: &Option<usize>,
        exclude: Option<Vec<String>>,
    ) -> Self {
        Self {
            min_chars: min_chars.map(MinChars),
            min_count: min_count.map(MinCount),
            exclude: exclude.map(ExcludeWords),
        }
    }

    /// Removes words from the `tally_map` based on any word `Filters`.
    pub fn apply(&self, tally_map: &mut IndexMap<Box<str>, usize>, case: Case) {
        if let Some(MinCount(min_count)) = self.min_count {
            tally_map.retain(|_, &mut count| count >= min_count);
        }

        if let Some(MinChars(min_chars)) = self.min_chars {
            tally_map.retain(|word, _| word.graphemes(true).count() >= min_chars);
        }

        if let Some(ExcludeWords(words)) = &self.exclude {
            let discard: HashSet<_> = words.iter().map(|word| case.normalize(word)).collect();
            tally_map.retain(|word, _| !discard.contains(word));
        }
    }
}

/// Minimum number of characters a word needs to have to be tallied.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
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

/// Minimum number of times a word needs to appear to be tallied.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct MinCount(pub usize);

impl Display for MinCount {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<usize> for MinCount {
    fn from(raw: usize) -> Self {
        Self(raw)
    }
}

/// A list of words that should be omitted from the tally.
#[derive(Clone, Debug, Default, Eq, PartialEq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ExcludeWords(pub Vec<String>);

impl Display for ExcludeWords {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.join(","))
    }
}

impl From<Vec<String>> for ExcludeWords {
    fn from(raw: Vec<String>) -> Self {
        Self(raw)
    }
}
