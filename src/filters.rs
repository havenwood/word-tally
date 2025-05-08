use crate::{Case, TallyMap};
use core::fmt::{self, Display, Formatter};
use core::ops::Deref;
use regex::Regex;
use std::collections::HashSet;
use unicode_segmentation::UnicodeSegmentation;

use serde::{Deserialize, Serialize};

/// Filters for which words should be tallied.
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Filters {
    /// Minimum characters required for a word.
    min_chars: Option<MinChars>,

    /// Minimum count for number of times a word must appear.
    min_count: Option<MinCount>,

    /// List of specific words to exclude.
    exclude_words: Option<ExcludeWords>,

    /// List of regex patterns to exclude words matching the patterns.
    #[serde(rename = "excludePatterns")]
    exclude_patterns: Option<ExcludePatterns>,

    /// List of regex patterns to include only words matching the patterns.
    #[serde(rename = "includePatterns")]
    include_patterns: Option<IncludePatterns>,
}

impl Filters {
    /// Constructs a new `Filters` instance with the specified filters.
    ///
    /// # Arguments
    ///
    /// * `min_chars` - Minimum characters per word
    /// * `min_count` - Minimum occurrences required
    /// * `exclude_words` - Words to exclude
    /// * `exclude_patterns` - Regex patterns to exclude
    /// * `include_patterns` - Regex patterns to include
    ///
    /// # Errors
    ///
    /// Returns a `regex::Error` if any of the provided patterns cannot be compiled into valid regular expressions.
    pub fn new(
        min_chars: &Option<usize>,
        min_count: &Option<usize>,
        exclude_words: Option<&Vec<String>>,
        exclude_patterns: Option<&Vec<String>>,
        include_patterns: Option<&Vec<String>>,
    ) -> Result<Self, regex::Error> {
        // Create initial filters
        let mut filters = Self {
            min_chars: min_chars.map(MinValue::new),
            min_count: min_count.map(MinValue::new),
            exclude_words: exclude_words
                .map(|words| ExcludeWords(words.iter().map(ToString::to_string).collect())),
            exclude_patterns: None,
            include_patterns: None,
        };

        // Add exclude regex patterns if provided
        if let Some(patterns) = exclude_patterns {
            if !patterns.is_empty() {
                filters = filters.with_exclude_patterns(patterns)?;
            }
        }

        // Add include regex patterns if provided
        if let Some(patterns) = include_patterns {
            if !patterns.is_empty() {
                filters = filters.with_include_patterns(patterns)?;
            }
        }

        Ok(filters)
    }

    /// Constructs an empty `Filters` instance.
    pub const fn empty() -> Self {
        Self {
            min_chars: None,
            min_count: None,
            exclude_words: None,
            exclude_patterns: None,
            include_patterns: None,
        }
    }

    /// Set minimum character requirement.
    pub const fn with_min_chars(mut self, min_chars: usize) -> Self {
        self.min_chars = Some(MinValue::new(min_chars));
        self
    }

    /// Set minimum count requirement.
    pub const fn with_min_count(mut self, min_count: usize) -> Self {
        self.min_count = Some(MinValue::new(min_count));
        self
    }

    /// Set words to exclude.
    pub fn with_exclude_words(mut self, words: Vec<String>) -> Self {
        self.exclude_words = Some(ExcludeWords(words));
        self
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

    /// Get the minimum character requirement
    pub const fn min_chars(&self) -> &Option<MinChars> {
        &self.min_chars
    }

    /// Get the minimum count requirement
    pub const fn min_count(&self) -> &Option<MinCount> {
        &self.min_count
    }

    /// Get the excluded words list
    pub const fn exclude_words(&self) -> &Option<ExcludeWords> {
        &self.exclude_words
    }

    /// Get the excluded patterns
    pub const fn exclude_patterns(&self) -> &Option<ExcludePatterns> {
        &self.exclude_patterns
    }

    /// Get the included patterns
    pub const fn include_patterns(&self) -> &Option<IncludePatterns> {
        &self.include_patterns
    }

    /// Removes words from the `tally_map` based on any word `Filters`.
    pub fn apply(&self, tally_map: &mut TallyMap, case: Case) {
        if let Some(min_count) = self.min_count() {
            let min_count_val = min_count.value;
            tally_map.retain(|_, &mut count| count >= min_count_val);
        }

        if let Some(min_chars) = self.min_chars() {
            let min_chars_val = min_chars.value;
            tally_map.retain(|word, _| word.graphemes(true).count() >= min_chars_val);
        }

        if let Some(ExcludeWords(words)) = self.exclude_words() {
            let discard: HashSet<_> = words.iter().map(|word| case.normalize(word)).collect();
            tally_map.retain(|word, _| !discard.contains(word));
        }

        if let Some(patterns) = self.exclude_patterns() {
            tally_map.retain(|word, _| !patterns.matches(word));
        }

        if let Some(patterns) = self.include_patterns() {
            tally_map.retain(|word, _| patterns.matches(word));
        }
    }
}

/// Generic wrapper for minimum value requirements.
#[derive(
    Clone, Copy, Debug, Default, Eq, PartialEq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct MinValue<T> {
    pub value: T,
}

impl<T> MinValue<T> {
    pub const fn new(value: T) -> Self {
        Self { value }
    }
}

impl<T: Display> Display for MinValue<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl<T> From<T> for MinValue<T> {
    fn from(value: T) -> Self {
        Self { value }
    }
}

impl<T> AsRef<T> for MinValue<T> {
    fn as_ref(&self) -> &T {
        &self.value
    }
}

impl From<MinValue<Self>> for usize {
    fn from(val: MinValue<Self>) -> Self {
        val.value
    }
}

/// Minimum number of characters a word needs to have to be tallied.
pub type MinChars = MinValue<usize>;

/// Minimum number of times a word needs to appear to be tallied.
pub type MinCount = MinValue<usize>;

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

impl AsRef<Vec<String>> for ExcludeWords {
    fn as_ref(&self) -> &Vec<String> {
        &self.0
    }
}

impl Deref for ExcludeWords {
    type Target = Vec<String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Base struct for regex pattern filtering.
#[derive(Clone, Debug, Default)]
struct Patterns {
    patterns: Vec<Regex>,
    original_patterns: Vec<String>,
}

impl Patterns {
    fn new(patterns: &[String]) -> Result<Self, regex::Error> {
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

    fn matches(&self, word: &str) -> bool {
        self.patterns.iter().any(|p| p.is_match(word))
    }
}

impl PartialEq for Patterns {
    fn eq(&self, other: &Self) -> bool {
        self.original_patterns == other.original_patterns
    }
}

impl Eq for Patterns {}

impl PartialOrd for Patterns {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Patterns {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.original_patterns.cmp(&other.original_patterns)
    }
}

impl std::hash::Hash for Patterns {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.original_patterns.hash(state);
    }
}

impl Display for Patterns {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.original_patterns.join(","))
    }
}

/// Regex patterns used to exclude matching words.
#[derive(Clone, Debug, Default)]
pub struct ExcludePatterns(Patterns);

impl PartialEq for ExcludePatterns {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for ExcludePatterns {}

impl PartialOrd for ExcludePatterns {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ExcludePatterns {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl std::hash::Hash for ExcludePatterns {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl Display for ExcludePatterns {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Serialize for ExcludePatterns {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.original_patterns.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ExcludePatterns {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;
        let patterns: Vec<String> = Vec::deserialize(deserializer)?;

        Self::new(&patterns).map_err(|e| D::Error::custom(format!("Error compiling regex: {}", e)))
    }
}

impl ExcludePatterns {
    pub fn new(patterns: &[String]) -> Result<Self, regex::Error> {
        Ok(Self(Patterns::new(patterns)?))
    }

    pub fn matches(&self, word: &str) -> bool {
        self.0.matches(word)
    }
}

/// Regex patterns used to include only matching words.
#[derive(Clone, Debug, Default)]
pub struct IncludePatterns(Patterns);

impl PartialEq for IncludePatterns {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for IncludePatterns {}

impl PartialOrd for IncludePatterns {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for IncludePatterns {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl std::hash::Hash for IncludePatterns {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl Display for IncludePatterns {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Serialize for IncludePatterns {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.original_patterns.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for IncludePatterns {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;
        let patterns: Vec<String> = Vec::deserialize(deserializer)?;

        Self::new(&patterns).map_err(|e| D::Error::custom(format!("Error compiling regex: {}", e)))
    }
}

impl IncludePatterns {
    pub fn new(patterns: &[String]) -> Result<Self, regex::Error> {
        Ok(Self(Patterns::new(patterns)?))
    }

    pub fn matches(&self, word: &str) -> bool {
        self.0.matches(word)
    }
}
