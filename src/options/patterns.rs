//! Regular expression pattern matching for word filtering.

use core::{
    cmp::Ordering,
    fmt::{self, Display, Formatter},
    hash::{Hash, Hasher},
};

use regex::RegexSet;
use serde::{Deserialize, Serialize, de::Error};

use crate::WordTallyError;

/// Collection of regex pattern strings.
pub type PatternList = Vec<String>;

/// Base struct for regex pattern filtering.
///
/// Contains a `Vec` of raw regexp input `String`s and their compiled `RegexSet`.
#[derive(Clone, Debug)]
struct Patterns {
    /// Original pattern strings.
    input_patterns: PatternList,
    /// Compiled regex set for matching.
    regex_set: RegexSet,
}

impl Default for Patterns {
    fn default() -> Self {
        Self {
            input_patterns: Vec::new(),
            regex_set: RegexSet::empty(),
        }
    }
}

impl PartialEq for Patterns {
    fn eq(&self, other: &Self) -> bool {
        self.input_patterns == other.input_patterns
    }
}

impl Eq for Patterns {}

impl PartialOrd for Patterns {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Patterns {
    fn cmp(&self, other: &Self) -> Ordering {
        self.input_patterns.cmp(&other.input_patterns)
    }
}

impl Hash for Patterns {
    fn hash<H: Hasher>(&self, state: &mut H) {
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
    /// Creates a pattern set and compiles the `RegexSet`.
    fn new(input_patterns: PatternList) -> Result<Self, WordTallyError> {
        let regex_set = RegexSet::new(&input_patterns).map_err(|e| WordTallyError::Pattern {
            kind: "regex".to_string(),
            message: e.to_string(),
        })?;

        Ok(Self {
            input_patterns,
            regex_set,
        })
    }

    /// Creates a pattern set from a slice of strings.
    fn from_slice(input_patterns: &[String]) -> Result<Self, WordTallyError> {
        Self::new(input_patterns.to_vec())
    }

    /// Checks if a word matches any pattern in the `RegexSet`.
    fn matches(&self, word: &str) -> bool {
        self.regex_set.is_match(word)
    }

    /// Returns a slice of the original input patterns.
    #[allow(clippy::missing_const_for_fn)]
    // Make this const when `const_vec_string_slice` is fully stabilized.
    // Requires stable `Vec::as_slice` in const contexts (tracked in rust-lang/rust#129041).
    fn as_patterns(&self) -> &[String] {
        &self.input_patterns
    }
}

/// Regex patterns used to exclude matching words.
///
/// # Examples
///
/// ```
/// use word_tally::ExcludeSet;
///
/// // Create a pattern to exclude words ending with "ly"
/// let patterns = ExcludeSet::new(vec!["ly$".to_string()]).unwrap();
///
/// // Test matching
/// assert!(patterns.matches("quickly"));
/// assert!(!patterns.matches("quick"));
/// ```
#[derive(Clone, Debug, Default)]
pub struct ExcludeSet(Patterns);

impl ExcludeSet {
    /// Creates patterns from owned pattern strings.
    ///
    /// # Examples
    ///
    /// ```
    /// use word_tally::ExcludeSet;
    ///
    /// // Create pattern for excluding numeric words
    /// let patterns = ExcludeSet::new(vec![r"^\d+$".to_string()]).unwrap();
    /// assert_eq!(patterns.len(), 1);
    ///
    /// // Test empty patterns
    /// let empty = ExcludeSet::default();
    /// assert!(empty.is_empty());
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if any pattern cannot be compiled into a valid regular expression.
    pub fn new(input_patterns: PatternList) -> Result<Self, WordTallyError> {
        Ok(Self(Patterns::new(input_patterns)?))
    }

    /// Tests if a word matches any pattern.
    #[must_use]
    pub fn matches(&self, word: &str) -> bool {
        self.0.matches(word)
    }

    /// Returns a slice of the original pattern strings.
    #[must_use]
    pub fn as_patterns(&self) -> &[String] {
        self.0.as_patterns()
    }

    /// Returns the number of patterns.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.0.input_patterns.len()
    }

    /// Returns true if there are no patterns.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.0.input_patterns.is_empty()
    }
}

impl<'a> TryFrom<&'a [String]> for ExcludeSet {
    type Error = WordTallyError;

    fn try_from(input_patterns: &'a [String]) -> Result<Self, Self::Error> {
        Ok(Self(Patterns::from_slice(input_patterns)?))
    }
}

impl AsRef<[String]> for ExcludeSet {
    fn as_ref(&self) -> &[String] {
        self.0.as_ref()
    }
}

impl Serialize for ExcludeSet {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.input_patterns.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ExcludeSet {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let input_patterns: PatternList = Vec::deserialize(deserializer)?;

        Self::new(input_patterns)
            .map_err(|e| D::Error::custom(format!("failed to compile exclude regex patterns: {e}")))
    }
}

impl PartialEq for ExcludeSet {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for ExcludeSet {}

impl PartialOrd for ExcludeSet {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ExcludeSet {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl Hash for ExcludeSet {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl Display for ExcludeSet {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Regex patterns used to include only matching words.
///
/// # Examples
///
/// ```
/// use word_tally::IncludeSet;
///
/// // Create a pattern to include only words containing vowels.
/// let patterns = IncludeSet::new(vec![r"[aeiou]".to_string()]).unwrap();
///
/// // Test matching
/// assert!(patterns.matches("test")); // Contains 'e'
/// assert!(!patterns.matches("rhythm")); // No vowels
/// ```
#[derive(Clone, Debug, Default)]
pub struct IncludeSet(Patterns);

impl IncludeSet {
    /// Creates patterns from owned pattern strings.
    ///
    /// # Examples
    ///
    /// ```
    /// use word_tally::IncludeSet;
    ///
    /// // Create patterns for including words with specific prefixes.
    /// let patterns = IncludeSet::new(vec![r"^pre".to_string(), r"^un".to_string()]).unwrap();
    ///
    /// assert_eq!(patterns.len(), 2);
    /// assert!(patterns.matches("prevent"));
    /// assert!(patterns.matches("unlike"));
    /// assert!(!patterns.matches("likely"));
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if any pattern cannot be compiled into a valid regular expression.
    pub fn new(input_patterns: PatternList) -> Result<Self, WordTallyError> {
        Ok(Self(Patterns::new(input_patterns)?))
    }

    /// Tests if a word matches any pattern.
    #[must_use]
    pub fn matches(&self, word: &str) -> bool {
        self.0.matches(word)
    }

    /// Returns a slice of the original pattern strings.
    #[must_use]
    pub fn as_patterns(&self) -> &[String] {
        self.0.as_patterns()
    }

    /// Returns the number of patterns.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.0.input_patterns.len()
    }

    /// Returns true if there are no patterns.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.0.input_patterns.is_empty()
    }
}

impl<'a> TryFrom<&'a [String]> for IncludeSet {
    type Error = WordTallyError;

    fn try_from(input_patterns: &'a [String]) -> Result<Self, Self::Error> {
        Ok(Self(Patterns::from_slice(input_patterns)?))
    }
}

impl AsRef<[String]> for IncludeSet {
    fn as_ref(&self) -> &[String] {
        self.0.as_ref()
    }
}

impl Serialize for IncludeSet {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.input_patterns.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for IncludeSet {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let input_patterns: PatternList = Vec::deserialize(deserializer)?;

        Self::new(input_patterns)
            .map_err(|e| D::Error::custom(format!("failed to compile include regex patterns: {e}")))
    }
}

impl PartialEq for IncludeSet {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for IncludeSet {}

impl PartialOrd for IncludeSet {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for IncludeSet {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl Hash for IncludeSet {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl Display for IncludeSet {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
