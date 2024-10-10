use crate::Case;
use core::fmt::{self, Display, Formatter};
use indexmap::IndexMap;
use unicode_segmentation::UnicodeSegmentation;

/// Filters for words to be included in the tally.
#[derive(Clone, Debug, Default, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct Filters {
    /// Word chars filters for tallying.
    pub min_chars: Option<MinChars>,

    /// Word count filters for tallying.
    pub min_count: Option<MinCount>,

    /// List of specific words to exclude for tallying.
    pub words_exclude: Option<WordsExclude>,

    /// List of specific words to only include for tallying.
    pub words_only: Option<WordsOnly>,
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

        // Remove any words on the `exclude` word list.
        if let Some(WordsExclude(excludes)) = &self.words_exclude {
            let normalized_excludes: Vec<_> = excludes
                .iter()
                .map(|exclude| case.apply_and_box(exclude))
                .collect();
            tally_map.retain(|word, _| !normalized_excludes.contains(word));
        }

        // Remove any words absent from the `only` word list.
        if let Some(WordsOnly(exclusives)) = &self.words_only {
            let normalized_exclusives: Vec<_> = exclusives
                .iter()
                .map(|exclusive| case.apply_and_box(exclusive))
                .collect();
            tally_map.retain(|word, _| normalized_exclusives.contains(word));
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
pub struct WordsExclude(pub Vec<String>);

impl From<Vec<String>> for WordsExclude {
    fn from(raw: Vec<String>) -> Self {
        Self(raw)
    }
}

/// A list of words that should only be tallied.
#[derive(Clone, Debug, Default, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct WordsOnly(pub Vec<String>);

impl From<Vec<String>> for WordsOnly {
    fn from(raw: Vec<String>) -> Self {
        Self(raw)
    }
}
