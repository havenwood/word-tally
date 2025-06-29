//! Word filtering based on length, frequency, patterns and exclusion lists.

use core::{
    fmt::{self, Display, Formatter},
    ops::Deref,
};

use anyhow::Result;
use hashbrown::HashSet;
use icu_segmenter::GraphemeClusterSegmenter;
use serde::{Deserialize, Serialize};

use crate::{
    Count, TallyMap,
    options::{
        case::Case,
        patterns::{ExcludeSet, IncludeSet, PatternList},
    },
};

/// Minimum number of characters a word needs to have to be tallied.
pub type MinChars = Count;

/// Minimum number of times a word needs to appear to be tallied.
pub type MinCount = Count;

/// Collection of words to be excluded from the tally.
pub type Words = Box<[Box<str>]>;

/// Filters for which words should be tallied.
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Filters {
    /// Minimum characters required for a word.
    min_chars: Option<MinChars>,

    /// Minimum count for number of times a word must appear.
    min_count: Option<MinCount>,

    /// List of specific words to exclude.
    exclude_words: Option<ExcludeWords>,

    /// List of regex patterns to exclude words matching the patterns.
    exclude_patterns: Option<ExcludeSet>,

    /// List of regex patterns to include only words matching the patterns.
    include_patterns: Option<IncludeSet>,
}

impl Filters {
    /// Constructs a new `Filters` instance with the specified filters.
    ///
    /// # Examples
    ///
    /// ```
    /// use word_tally::Filters;
    ///
    /// # fn example() -> anyhow::Result<()> {
    /// // Filter out common words and focus on content
    /// let filters = Filters::default()
    ///     .with_min_chars(4)
    ///     .with_exclude_words(vec![
    ///         "the".to_string(),
    ///         "and".to_string(),
    ///         "for".to_string(),
    ///         "with".to_string(),
    ///     ]);
    ///
    /// // Analyze only capitalized words (names, places)
    /// let name_filter = Filters::default().with_include_patterns(&vec!["^[A-Z]".to_string()])?;
    ///
    /// // Focus on frequently used terms
    /// let frequent = Filters::default().with_min_count(5).with_min_chars(3);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if any of the provided patterns cannot be compiled into valid regular
    /// expressions.
    pub fn new(
        min_chars: Option<MinChars>,
        min_count: Option<MinCount>,
        exclude_words: Option<Vec<String>>,
        exclude_patterns: Option<PatternList>,
        include_patterns: Option<PatternList>,
    ) -> Result<Self> {
        // Create initial filters
        let mut filters = Self {
            min_chars,
            min_count,
            exclude_words: exclude_words.map(ExcludeWords::from),
            exclude_patterns: None,
            include_patterns: None,
        };

        // Add exclude regex patterns if provided
        if let Some(patterns) = exclude_patterns
            && !patterns.is_empty()
        {
            filters = filters.with_exclude_patterns(&patterns)?;
        }

        // Add include regex patterns if provided
        if let Some(patterns) = include_patterns
            && !patterns.is_empty()
        {
            filters = filters.with_include_patterns(&patterns)?;
        }

        Ok(filters)
    }

    /// Set minimum character requirement.
    ///
    /// # Examples
    ///
    /// ```
    /// use word_tally::Filters;
    ///
    /// let filters = Filters::default().with_min_chars(3);
    /// assert_eq!(filters.min_chars(), Some(3));
    /// ```
    #[must_use]
    pub const fn with_min_chars(mut self, min_chars: MinChars) -> Self {
        self.min_chars = Some(min_chars);
        self
    }

    /// Set minimum count requirement.
    #[must_use]
    pub const fn with_min_count(mut self, min_count: MinCount) -> Self {
        self.min_count = Some(min_count);
        self
    }

    /// Set words to exclude.
    #[must_use]
    pub fn with_exclude_words(mut self, words: Vec<String>) -> Self {
        self.exclude_words = Some(words.into());
        self
    }

    /// Helper method to set patterns on filters.
    fn with_patterns<T>(
        mut self,
        input_patterns: &[String],
        setter: impl FnOnce(&mut Self, Option<T>),
        converter: impl FnOnce(&[String]) -> Result<T>,
    ) -> Result<Self> {
        let pattern = if input_patterns.is_empty() {
            None
        } else {
            Some(converter(input_patterns)?)
        };

        setter(&mut self, pattern);

        Ok(self)
    }

    /// Sets patterns to exclude words that match.
    ///
    /// # Errors
    ///
    /// Returns an error if any pattern cannot be compiled into a valid regular expression.
    pub fn with_exclude_patterns(self, input_patterns: &PatternList) -> Result<Self> {
        self.with_patterns(
            input_patterns,
            |s, p| s.exclude_patterns = p,
            |patterns| Ok(ExcludeSet::new(patterns.to_vec())?),
        )
    }

    /// Sets patterns to include only words that match.
    ///
    /// # Errors
    ///
    /// Returns an error if any pattern cannot be compiled into a valid regular expression.
    pub fn with_include_patterns(self, input_patterns: &PatternList) -> Result<Self> {
        self.with_patterns(
            input_patterns,
            |s, p| s.include_patterns = p,
            |patterns| Ok(IncludeSet::new(patterns.to_vec())?),
        )
    }

    /// Get the minimum character requirement.
    #[must_use]
    pub const fn min_chars(&self) -> Option<MinChars> {
        self.min_chars
    }

    /// Get the minimum count requirement
    #[must_use]
    pub const fn min_count(&self) -> Option<MinCount> {
        self.min_count
    }

    /// Get the excluded words list
    #[must_use]
    pub const fn exclude_words(&self) -> Option<&ExcludeWords> {
        self.exclude_words.as_ref()
    }

    /// Get the excluded patterns.
    #[must_use]
    pub const fn exclude_patterns(&self) -> Option<&ExcludeSet> {
        self.exclude_patterns.as_ref()
    }

    /// Get the included patterns.
    #[must_use]
    pub const fn include_patterns(&self) -> Option<&IncludeSet> {
        self.include_patterns.as_ref()
    }

    /// Removes words from the `tally_map` based on any word `Filters`.
    pub fn apply(&self, tally_map: &mut TallyMap, case: Case) {
        if let Some(min_count) = self.min_count() {
            tally_map.retain(|_, &mut count| count >= min_count);
        }

        if let Some(min_chars) = self.min_chars() {
            let segmenter = GraphemeClusterSegmenter::new();
            tally_map.retain(|word, _| segmenter.segment_str(word).count() > min_chars);
        }

        if let Some(exclude_words) = self.exclude_words() {
            let discard: HashSet<Box<str>> = exclude_words
                .iter()
                .map(|word| case.normalize_unicode(word).into())
                .collect();
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
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ExcludeWords(Words);

impl Display for ExcludeWords {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            self.0
                .iter()
                .map(AsRef::as_ref)
                .collect::<Vec<_>>()
                .join(",")
        )
    }
}

impl From<Vec<String>> for ExcludeWords {
    fn from(strings: Vec<String>) -> Self {
        let words: Box<[Box<str>]> = strings.into_iter().map(Into::into).collect();
        Self(words)
    }
}

impl From<Words> for ExcludeWords {
    fn from(raw: Words) -> Self {
        Self(raw)
    }
}

impl AsRef<Words> for ExcludeWords {
    fn as_ref(&self) -> &Words {
        &self.0
    }
}

impl Deref for ExcludeWords {
    type Target = Words;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
