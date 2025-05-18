//! Serialization format options and settings.

use anyhow::{Context, Result};
use clap::ValueEnum;
use core::fmt::{self, Display, Formatter};
use serde::{self, Deserialize, Serialize};
use unescaper::unescape;

/// Default delimiter used between word and count in text output.
pub const DEFAULT_DELIMITER: &str = " ";

/// Serialization format options.
///
/// # Examples
///
/// ```
/// use word_tally::Format;
///
/// assert_eq!(Format::default(), Format::Text);
/// assert_eq!(Format::Json.to_string(), "json");
/// ```
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Eq,
    PartialEq,
    PartialOrd,
    Ord,
    Hash,
    ValueEnum,
    Serialize,
    Deserialize,
)]
pub enum Format {
    #[default]
    Text,
    Json,
    Csv,
}

impl Display for Format {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Text => write!(f, "text"),
            Self::Json => write!(f, "json"),
            Self::Csv => write!(f, "csv"),
        }
    }
}

/// Serialization settings for word tallying.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Serialization {
    /// Output serialization format (text, json, csv)
    pub format: Format,

    /// Delimiter between word and count in text output
    pub delimiter: String,
}

impl Default for Serialization {
    fn default() -> Self {
        Self {
            format: Format::default(),
            delimiter: DEFAULT_DELIMITER.to_string(),
        }
    }
}

impl Display for Serialization {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Serialization {{ format: {}, delimiter: \"{}\" }}",
            self.format, self.delimiter
        )
    }
}

impl Serialization {
    /// Helper function to unescape a delimiter string
    fn format_delimiter(delimiter: &str) -> Result<String> {
        unescape(delimiter).with_context(|| format!("failed to unescape delimiter: {}", delimiter))
    }

    /// Create a new Serialize instance with specified settings
    pub fn new(format: Format, delimiter: &str) -> Result<Self> {
        let formatted_delimiter = Self::format_delimiter(delimiter)?;

        Ok(Self {
            format,
            delimiter: formatted_delimiter,
        })
    }

    /// Create a Serialize with custom format
    pub fn with_format(format: Format) -> Self {
        Self {
            format,
            ..Default::default()
        }
    }

    /// Create a Serialize with custom delimiter
    pub fn with_delimiter(delimiter: &str) -> Result<Self> {
        let formatted_delimiter = Self::format_delimiter(delimiter)?;

        Ok(Self {
            delimiter: formatted_delimiter,
            ..Default::default()
        })
    }

    /// Set the format option and return a new instance
    ///
    /// # Examples
    ///
    /// ```
    /// use word_tally::{Format, Serialization};
    ///
    /// let serialization = Serialization::default().with_format_setting(Format::Json);
    /// assert_eq!(serialization.format(), Format::Json);
    /// ```
    pub const fn with_format_setting(mut self, format: Format) -> Self {
        self.format = format;
        self
    }

    /// Get the format setting
    pub const fn format(&self) -> Format {
        self.format
    }

    /// Get the delimiter
    pub fn delimiter(&self) -> &str {
        &self.delimiter
    }
}
