use crate::Case;
use core::fmt::{self, Display, Formatter};
use indexmap::IndexMap;
use regex::Regex;
use std::collections::HashSet;
use unicode_segmentation::UnicodeSegmentation;

use serde::{Deserialize, Serialize};

/// Filters for which words should be tallied.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Filters {
    /// Minimum characters required for a word.
    pub min_chars: Option<MinChars>,

    /// Minimum count for number of times a word must appear.
    pub min_count: Option<MinCount>,

    /// List of specific words to exclude.
    pub exclude_words: Option<ExcludeWords>,

    /// List of regex patterns to exclude words matching the patterns.
    #[serde(skip)]
    pub exclude_patterns: Option<ExcludePatterns>,

    /// List of regex patterns to include only words matching the patterns.
    #[serde(skip)]
    pub include_patterns: Option<IncludePatterns>,
}

// Manual implementations to ignore exclude_patterns and include_patterns fields
impl PartialEq for Filters {
    fn eq(&self, other: &Self) -> bool {
        self.min_chars == other.min_chars &&
        self.min_count == other.min_count &&
        self.exclude_words == other.exclude_words
    }
}

impl Eq for Filters {}

impl PartialOrd for Filters {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Filters {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self.min_chars.cmp(&other.min_chars),
               self.min_count.cmp(&other.min_count)) {
            (core::cmp::Ordering::Equal, core::cmp::Ordering::Equal) => {
                self.exclude_words.cmp(&other.exclude_words)
            }
            (core::cmp::Ordering::Equal, ord) => ord,
            (ord, _) => ord,
        }
    }
}

impl std::hash::Hash for Filters {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.min_chars.hash(state);
        self.min_count.hash(state);
        self.exclude_words.hash(state);
    }
}

impl Filters {
    /// Constructs `Filters`.
    pub fn new(
        min_chars: &Option<usize>,
        min_count: &Option<usize>,
        exclude_words: Option<Vec<String>>,
    ) -> Self {
        Self {
            min_chars: min_chars.map(MinChars),
            min_count: min_count.map(MinCount),
            exclude_words: exclude_words.map(ExcludeWords),
            exclude_patterns: None,
            include_patterns: None,
        }
    }

    /// Sets exclude patterns on an existing `Filters` instance.
    pub fn with_exclude_patterns(mut self, patterns: &[String]) -> Result<Self, regex::Error> {
        if patterns.is_empty() {
            self.exclude_patterns = None;
        } else {
            self.exclude_patterns = Some(ExcludePatterns::new(patterns)?);
        }
        Ok(self)
    }

    /// Sets include patterns on an existing `Filters` instance.
    pub fn with_include_patterns(mut self, patterns: &[String]) -> Result<Self, regex::Error> {
        if patterns.is_empty() {
            self.include_patterns = None;
        } else {
            self.include_patterns = Some(IncludePatterns::new(patterns)?);
        }
        Ok(self)
    }

    /// Removes words from the `tally_map` based on any word `Filters`.
    pub fn apply(&self, tally_map: &mut IndexMap<Box<str>, usize>, case: Case) {
        if let Some(MinCount(min_count)) = self.min_count {
            tally_map.retain(|_, &mut count| count >= min_count);
        }

        if let Some(MinChars(min_chars)) = self.min_chars {
            tally_map.retain(|word, _| word.graphemes(true).count() >= min_chars);
        }

        if let Some(ExcludeWords(words)) = &self.exclude_words {
            let discard: HashSet<_> = words.iter().map(|word| case.normalize(word)).collect();
            tally_map.retain(|word, _| !discard.contains(word));
        }

        if let Some(patterns) = &self.exclude_patterns {
            tally_map.retain(|word, _| !patterns.matches(word));
        }

        if let Some(patterns) = &self.include_patterns {
            tally_map.retain(|word, _| patterns.matches(word));
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

/// A collection of regex patterns used to exclude words that match.
///
/// Each pattern is compiled into a Regex and used to filter out words
/// whose text matches any of the patterns.
#[derive(Clone, Debug)]
pub struct ExcludePatterns {
    /// Compiled regex patterns.
    patterns: Vec<Regex>,
    /// Original pattern strings (for display purposes).
    original_patterns: Vec<String>,
}

impl ExcludePatterns {
    /// Constructs a new `ExcludePatterns` from a slice of pattern strings.
    ///
    /// Returns an error if any of the patterns fail to compile as a valid regex.
    pub fn new(patterns: &[String]) -> Result<Self, regex::Error> {
        let original_patterns = patterns.to_vec();
        let compiled_patterns = patterns
            .iter()
            .map(|p| Regex::new(p))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            patterns: compiled_patterns,
            original_patterns,
        })
    }

    /// Checks if a word matches any of the patterns.
    ///
    /// Returns `true` if the word matches any pattern, `false` otherwise.
    pub fn matches(&self, word: &str) -> bool {
        self.patterns.iter().any(|p| p.is_match(word))
    }
}

impl Display for ExcludePatterns {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.original_patterns.join(","))
    }
}

/// A collection of regex patterns used to include only words that match.
///
/// Each pattern is compiled into a Regex and used to filter in only words
/// whose text matches any of the patterns.
#[derive(Clone, Debug)]
pub struct IncludePatterns {
    /// Compiled regex patterns.
    patterns: Vec<Regex>,
    /// Original pattern strings (for display purposes).
    original_patterns: Vec<String>,
}

impl IncludePatterns {
    /// Constructs a new `IncludePatterns` from a slice of pattern strings.
    ///
    /// Returns an error if any of the patterns fail to compile as a valid regex.
    pub fn new(patterns: &[String]) -> Result<Self, regex::Error> {
        let original_patterns = patterns.to_vec();
        let compiled_patterns = patterns
            .iter()
            .map(|p| Regex::new(p))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            patterns: compiled_patterns,
            original_patterns,
        })
    }

    /// Checks if a word matches any of the patterns.
    ///
    /// Returns `true` if the word matches any pattern, `false` otherwise.
    pub fn matches(&self, word: &str) -> bool {
        self.patterns.iter().any(|p| p.is_match(word))
    }
}

impl Display for IncludePatterns {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.original_patterns.join(","))
    }
}
