//! Delimiters for text serialization.

use core::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

/// A delimiter separates text serialization words/counts or entries.
pub type Delimiter = String;

/// Delimiters for text format serialization.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Delimiters {
    field: Delimiter,
    entry: Delimiter,
}

impl Delimiters {
    /// Default delimiter between field and value.
    pub const DEFAULT_FIELD: &str = " ";
    /// Default delimiter between entries.
    pub const DEFAULT_ENTRY: &str = "\n";

    /// Create delimiters with specified field and entry separators.
    #[must_use]
    pub fn new(field: &str, entry: &str) -> Self {
        Self {
            field: field.to_string(),
            entry: entry.to_string(),
        }
    }

    /// Set the field delimiter.
    #[must_use]
    pub fn with_field_delimiter(mut self, delimiter: &str) -> Self {
        self.field = delimiter.to_string();
        self
    }

    /// Set the entry delimiter.
    #[must_use]
    pub fn with_entry_delimiter(mut self, delimiter: &str) -> Self {
        self.entry = delimiter.to_string();
        self
    }

    /// Get the field delimiter.
    #[must_use]
    pub fn field(&self) -> &str {
        &self.field
    }

    /// Get the entry delimiter.
    #[must_use]
    pub fn entry(&self) -> &str {
        &self.entry
    }

    /// Get the field delimiter formatted for display.
    #[must_use]
    pub fn field_display(&self) -> String {
        format!("{:?}", self.field)
    }

    /// Get the entry delimiter formatted for display.
    #[must_use]
    pub fn entry_display(&self) -> String {
        format!("{:?}", self.entry)
    }
}

impl Default for Delimiters {
    fn default() -> Self {
        Self {
            field: Self::DEFAULT_FIELD.to_string(),
            entry: Self::DEFAULT_ENTRY.to_string(),
        }
    }
}

impl Display for Delimiters {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "field={:?}, entry={:?}", self.field, self.entry)
    }
}
