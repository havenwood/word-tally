//! Serialization format options and settings.
//!
//! Word tallies can be serialized in three formats:
//! - **Text**: Customizable delimiters for field/value and entry separation
//! - **JSON**: Machine-readable structured format
//! - **CSV**: Comma-separated values for spreadsheet compatibility
//!
//! # Delimiter Configuration
//!
//! Delimiters are only applicable to text format. When using JSON or CSV formats
//! with the CLI, delimiter flags (`--field-delimiter`, `--entry-delimiter`) will
//! cause an error. For the library API, delimiters are encapsulated within the
//! `Text` variant and cannot be mistakenly applied to other formats.
//!
//! # Examples
//!
//! ```
//! use word_tally::{Delimiters, Serialization};
//!
//! // Text format with custom delimiters
//! let text_format = Serialization::Text(
//!     Delimiters::default()
//!         .with_field_delimiter("::")
//!         .with_entry_delimiter(";")
//! );
//!
//! // JSON format (no delimiter configuration)
//! let json_format = Serialization::Json;
//!
//! // CSV format (uses standard CSV delimiters)
//! let csv_format = Serialization::Csv;
//! ```

use core::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::options::delimiters::Delimiters;

/// Serialization format options.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Serialization {
    /// Plain text output with configurable delimiters.
    Text(Delimiters),
    /// JSON output.
    Json,
    /// CSV output.
    Csv,
}

impl Serialization {
    /// Get the field delimiter formatted for display, or "n/a" if not text format.
    #[must_use]
    pub fn field_delimiter_display(&self) -> String {
        match self {
            Self::Text(delimiters) => delimiters.field_display(),
            _ => "n/a".to_string(),
        }
    }

    /// Get the entry delimiter formatted for display, or "n/a" if not text format.
    #[must_use]
    pub fn entry_delimiter_display(&self) -> String {
        match self {
            Self::Text(delimiters) => delimiters.entry_display(),
            _ => "n/a".to_string(),
        }
    }
}

impl Default for Serialization {
    fn default() -> Self {
        Self::Text(Delimiters::default())
    }
}

impl Display for Serialization {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Text(delimiters) => write!(f, "text[{delimiters}]"),
            Self::Json => write!(f, "json"),
            Self::Csv => write!(f, "csv"),
        }
    }
}
