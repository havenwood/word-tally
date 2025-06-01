//! Serialization format options and settings.

use crate::options::delimiter::Delimiter;
use core::fmt::{self, Display, Formatter};
use serde::{Deserialize, Serialize};

/// Serialization format options.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Serialization {
    /// Plain text output has a delimiter between field/value and a per-entry delimiter.
    Text {
        field_delimiter: Delimiter,
        entry_delimiter: Delimiter,
    },
    /// JSON output.
    Json,
    /// CSV output.
    Csv,
}

impl Serialization {
    /// Create a text format with default delimiters.
    pub fn text() -> Self {
        Self::Text {
            field_delimiter: Delimiter::from_literal(Delimiter::DEFAULT_FIELD),
            entry_delimiter: Delimiter::from_literal(Delimiter::DEFAULT_ENTRY),
        }
    }

    /// Set the field delimiter if this is a text format, otherwise return self unchanged.
    pub fn with_field_delimiter(self, field_delimiter: &str) -> Self {
        if let Self::Text {
            entry_delimiter, ..
        } = self
        {
            Self::Text {
                field_delimiter: Delimiter::from_escaped(field_delimiter),
                entry_delimiter,
            }
        } else {
            self
        }
    }

    /// Set the entry delimiter if this is a text format, otherwise return self unchanged.
    pub fn with_entry_delimiter(self, entry_delimiter: &str) -> Self {
        if let Self::Text {
            field_delimiter, ..
        } = self
        {
            Self::Text {
                field_delimiter,
                entry_delimiter: Delimiter::from_escaped(entry_delimiter),
            }
        } else {
            self
        }
    }

    /// Get the field delimiter if this is text format.
    #[must_use]
    pub fn field_delimiter(&self) -> Option<&str> {
        if let Self::Text {
            field_delimiter, ..
        } = self
        {
            Some(field_delimiter.as_str())
        } else {
            None
        }
    }

    /// Get the entry delimiter if this is text format.
    #[must_use]
    pub fn entry_delimiter(&self) -> Option<&str> {
        if let Self::Text {
            entry_delimiter, ..
        } = self
        {
            Some(entry_delimiter.as_str())
        } else {
            None
        }
    }

    /// Get the field delimiter formatted for display, or "n/a" if not text format.
    #[must_use]
    pub fn field_delimiter_display(&self) -> String {
        if let Self::Text {
            field_delimiter, ..
        } = self
        {
            field_delimiter.display_quoted()
        } else {
            "n/a".to_string()
        }
    }

    /// Get the entry delimiter formatted for display, or "n/a" if not text format.
    #[must_use]
    pub fn entry_delimiter_display(&self) -> String {
        if let Self::Text {
            entry_delimiter, ..
        } = self
        {
            entry_delimiter.display_quoted()
        } else {
            "n/a".to_string()
        }
    }
}

impl Default for Serialization {
    fn default() -> Self {
        Self::text()
    }
}

impl Display for Serialization {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Text {
                field_delimiter,
                entry_delimiter,
            } => {
                write!(
                    f,
                    "text[field={}, entry={}]",
                    field_delimiter, entry_delimiter
                )
            }
            Self::Json => write!(f, "json"),
            Self::Csv => write!(f, "csv"),
        }
    }
}
