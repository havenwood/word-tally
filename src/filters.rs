use crate::{Case, Count, TallyMap};
use bstr::ByteSlice;
use core::fmt::{self, Display, Formatter};
use core::ops::Deref;
use regex::RegexSet;
use std::collections::HashSet;

use serde::{Deserialize, Serialize};

/// Collection of regex pattern strings.
type InputPatterns = Vec<String>;

/// Minimum number of characters a word needs to have to be tallied.
pub type MinChars = Count;

/// Minimum number of times a word needs to appear to be tallied.
pub type MinCount = Count;

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
        min_chars: &Option<MinChars>,
        min_count: &Option<MinCount>,
        exclude_words: Option<&Vec<String>>,
        exclude_patterns: Option<&Vec<String>>,
        include_patterns: Option<&Vec<String>>,
    ) -> Result<Self, regex::Error> {
        // Create initial filters
        let mut filters = Self {
            min_chars: *min_chars,
            min_count: *min_count,
            exclude_words: exclude_words.map(|words| ExcludeWords::from(words.to_vec())),
            exclude_patterns: None,
            include_patterns: None,
        };

        // Add exclude regex patterns if provided
        if let Some(patterns) = exclude_patterns.filter(|p| !p.is_empty()) {
            filters = filters.with_exclude_patterns(patterns)?;
        }

        // Add include regex patterns if provided
        if let Some(patterns) = include_patterns.filter(|p| !p.is_empty()) {
            filters = filters.with_include_patterns(patterns)?;
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
    pub const fn with_min_chars(mut self, min_chars: MinChars) -> Self {
        self.min_chars = Some(min_chars);
        self
    }

    /// Set minimum count requirement.
    pub const fn with_min_count(mut self, min_count: MinCount) -> Self {
        self.min_count = Some(min_count);
        self
    }

    /// Set words to exclude.
    pub fn with_exclude_words(mut self, words: Vec<String>) -> Self {
        self.exclude_words = Some(ExcludeWords(words));
        self
    }

    /// Helper method to set patterns on filters
    fn with_patterns<T>(
        mut self,
        input_patterns: &[String],
        setter: impl FnOnce(&mut Self, Option<T>),
        converter: impl FnOnce(&[String]) -> Result<T, regex::Error>,
    ) -> Result<Self, regex::Error> {
        let pattern = if input_patterns.is_empty() {
            None
        } else {
            Some(converter(input_patterns)?)
        };

        setter(&mut self, pattern);
        Ok(self)
    }

    /// Sets patterns to exclude words that match.
    pub fn with_exclude_patterns(self, input_patterns: &[String]) -> Result<Self, regex::Error> {
        self.with_patterns(
            input_patterns,
            |s, p| s.exclude_patterns = p,
            |patterns| patterns.try_into(),
        )
    }

    /// Sets patterns to include only words that match.
    pub fn with_include_patterns(self, input_patterns: &[String]) -> Result<Self, regex::Error> {
        self.with_patterns(
            input_patterns,
            |s, p| s.include_patterns = p,
            |patterns| patterns.try_into(),
        )
    }

    /// Get the minimum character requirement
    pub const fn min_chars(&self) -> Option<MinChars> {
        self.min_chars
    }

    /// Get the minimum count requirement
    pub const fn min_count(&self) -> Option<MinCount> {
        self.min_count
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
            tally_map.retain(|_, &mut count| count >= min_count);
        }

        if let Some(min_chars) = self.min_chars() {
            tally_map.retain(|word, _| word.as_bytes().graphemes().count() >= min_chars);
        }

        if let Some(ExcludeWords(words)) = self.exclude_words() {
            let discard: HashSet<_> = words.iter().map(|word| case.normalize(word)).collect();
            tally_map.retain(|word, _| !discard.contains(word));
        }

        if let Some(input_patterns) = self.exclude_patterns() {
            tally_map.retain(|word, _| !input_patterns.matches(word));
        }

        if let Some(input_patterns) = self.include_patterns() {
            tally_map.retain(|word, _| input_patterns.matches(word));
        }
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
///
/// Contains a `Vec` of raw regexp input `String`s and their compiled `RegexSet`.
#[derive(Clone, Debug)]
struct Patterns {
    /// Original pattern strings
    input_patterns: InputPatterns,
    /// Compiled regex set for matching
    regex_set: RegexSet,
}

impl Default for Patterns {
    fn default() -> Self {
        // Use an empty patterns array and empty RegexSet
        Self::new(Vec::new()).expect("Default creation with empty vec should never fail")
    }
}

impl PartialEq for Patterns {
    fn eq(&self, other: &Self) -> bool {
        self.input_patterns == other.input_patterns
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
        self.input_patterns.cmp(&other.input_patterns)
    }
}

impl std::hash::Hash for Patterns {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.input_patterns.hash(state);
    }
}

impl Display for Patterns {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.input_patterns.join(","))
    }
}

impl AsRef<[String]> for Patterns {
    fn as_ref(&self) -> &[String] {
        &self.input_patterns
    }
}

impl Patterns {
    /// Creates a pattern set and compiles the RegexSet.
    fn new(input_patterns: InputPatterns) -> Result<Self, regex::Error> {
        let regex_set = RegexSet::new(&input_patterns)?;
        Ok(Self {
            regex_set,
            input_patterns,
        })
    }

    /// Creates a pattern set from a slice of strings.
    fn from_slice(input_patterns: &[String]) -> Result<Self, regex::Error> {
        Self::new(input_patterns.to_vec())
    }

    /// Checks if a word matches any pattern in the RegexSet.
    fn matches(&self, word: &str) -> bool {
        self.regex_set.is_match(word)
    }

    /// Returns a slice of the original input patterns.
    #[allow(clippy::missing_const_for_fn)]
    // Make this const and remove the `#allow` when `const_vec_string_slice` is fully stabilized.
    // Requires stable `Vec::as_slice` in const contexts (tracked in rust-lang/rust#129041).
    fn as_patterns(&self) -> &[String] {
        &self.input_patterns
    }
}

/// Regex patterns used to exclude matching words.
#[derive(Clone, Debug, Default)]
pub struct ExcludePatterns(Patterns);

impl ExcludePatterns {
    /// Creates patterns from owned pattern strings.
    pub fn new(input_patterns: InputPatterns) -> Result<Self, regex::Error> {
        Ok(Self(Patterns::new(input_patterns)?))
    }

    /// Tests if a word matches any pattern.
    pub fn matches(&self, word: &str) -> bool {
        self.0.matches(word)
    }

    /// Returns a slice of the original pattern strings.
    pub fn as_patterns(&self) -> &[String] {
        self.0.as_patterns()
    }

    /// Returns the number of patterns.
    pub fn len(&self) -> usize {
        self.0.input_patterns.len()
    }

    /// Returns true if there are no patterns.
    pub fn is_empty(&self) -> bool {
        self.0.input_patterns.is_empty()
    }
}

impl<'a> TryFrom<&'a [String]> for ExcludePatterns {
    type Error = regex::Error;

    fn try_from(input_patterns: &'a [String]) -> Result<Self, Self::Error> {
        Ok(Self(Patterns::from_slice(input_patterns)?))
    }
}

impl AsRef<[String]> for ExcludePatterns {
    fn as_ref(&self) -> &[String] {
        self.0.as_ref()
    }
}

impl Serialize for ExcludePatterns {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.input_patterns.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ExcludePatterns {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;
        let input_patterns: InputPatterns = Vec::deserialize(deserializer)?;

        Self::new(input_patterns)
            .map_err(|e| D::Error::custom(format!("Error compiling regex: {}", e)))
    }
}

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

/// Regex patterns used to include only matching words.
#[derive(Clone, Debug, Default)]
pub struct IncludePatterns(Patterns);

impl IncludePatterns {
    /// Creates patterns from owned pattern strings.
    pub fn new(input_patterns: InputPatterns) -> Result<Self, regex::Error> {
        Ok(Self(Patterns::new(input_patterns)?))
    }

    /// Tests if a word matches any pattern.
    pub fn matches(&self, word: &str) -> bool {
        self.0.matches(word)
    }

    /// Returns a slice of the original pattern strings.
    pub fn as_patterns(&self) -> &[String] {
        self.0.as_patterns()
    }

    /// Returns the number of patterns.
    pub fn len(&self) -> usize {
        self.0.input_patterns.len()
    }

    /// Returns true if there are no patterns.
    pub fn is_empty(&self) -> bool {
        self.0.input_patterns.is_empty()
    }
}

impl<'a> TryFrom<&'a [String]> for IncludePatterns {
    type Error = regex::Error;

    fn try_from(input_patterns: &'a [String]) -> Result<Self, Self::Error> {
        Ok(Self(Patterns::from_slice(input_patterns)?))
    }
}

impl AsRef<[String]> for IncludePatterns {
    fn as_ref(&self) -> &[String] {
        self.0.as_ref()
    }
}

impl Serialize for IncludePatterns {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.input_patterns.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for IncludePatterns {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;
        let input_patterns: InputPatterns = Vec::deserialize(deserializer)?;

        Self::new(input_patterns)
            .map_err(|e| D::Error::custom(format!("Error compiling regex: {}", e)))
    }
}

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
