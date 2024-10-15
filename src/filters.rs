use crate::Case;
use core::fmt::{self, Display, Formatter};
use indexmap::IndexMap;
use std::collections::HashSet;
use unicode_segmentation::UnicodeSegmentation;

/// Filters for words to be included in the tally.
#[derive(Clone, Debug, Default, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct Filters {
    /// Word chars filters for tallying.
    pub min_chars: Option<MinChars>,

    /// Word count filters for tallying.
    pub min_count: Option<MinCount>,

    /// List of specific words to exclude for tallying.
    pub exclude: Option<ExcludeWords>,
}

impl Filters {
    /// Removes words from the `tally_map` based on any word `Filters`.
    pub fn apply(&self, tally_map: &mut IndexMap<Box<str>, usize>, case: Case) {
        // Remove any words that lack the minimum count.
        if let Some(MinCount(min_count)) = self.min_count {
            tally_map.retain(|_, &mut count| count >= min_count);
        }

        // Remove any words that lack the minimum numbner of characters.
        if let Some(MinChars(min_chars)) = self.min_chars {
            tally_map.retain(|word, _| word.graphemes(true).count() >= min_chars);
        }

        // Remove any words on the `discard` list.
        if let Some(ExcludeWords(words)) = &self.exclude {
            let discard: HashSet<_> = words.iter().map(|word| case.apply_and_box(word)).collect();
            tally_map.retain(|word, _| !discard.contains(word));
        }
    }
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

/// A list of words that should not be tallied.
#[derive(Clone, Debug, Default, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct ExcludeWords(pub Vec<String>);

impl From<Vec<String>> for ExcludeWords {
    fn from(raw: Vec<String>) -> Self {
        Self(raw)
    }
}
