//! Delimiters for text serialization.

use core::{
    fmt::{self, Display, Formatter},
    ops::Deref,
};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

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
            field: field.into(),
            entry: entry.into(),
        }
    }

    /// Set the field delimiter.
    #[must_use]
    pub fn with_field_delimiter(mut self, delimiter: &str) -> Self {
        self.field = Delimiter::from(delimiter);
        self
    }

    /// Set the entry delimiter.
    #[must_use]
    pub fn with_entry_delimiter(mut self, delimiter: &str) -> Self {
        self.entry = Delimiter::from(delimiter);
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
        self.field.to_string()
    }

    /// Get the entry delimiter formatted for display.
    #[must_use]
    pub fn entry_display(&self) -> String {
        self.entry.to_string()
    }
}

impl Default for Delimiters {
    fn default() -> Self {
        Self {
            field: Delimiter::from(Self::DEFAULT_FIELD),
            entry: Delimiter::from(Self::DEFAULT_ENTRY),
        }
    }
}

impl Display for Delimiters {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "field={}, entry={}", self.field, self.entry)
    }
}

/// A delimiter separates text serialization words/counts or entries.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Delimiter(String);

impl Delimiter {
    /// Create a delimiter from a literal string.
    #[must_use]
    pub fn new(s: &str) -> Self {
        s.into()
    }
}

impl Display for Delimiter {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // Display quoted to ensure all delimiters are visible, including spaces
        write!(f, "{:?}", self.0)
    }
}

impl From<String> for Delimiter {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for Delimiter {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl Deref for Delimiter {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Serialize for Delimiter {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Delimiter {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        String::deserialize(deserializer).map(Self)
    }
}
