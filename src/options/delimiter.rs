//! Delimiters for text serialization.

use core::fmt::{self, Display, Formatter};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// A delimiter separates text serialization words/counts or entries.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Delimiter(String);

impl Delimiter {
    /// Default delimiter between field and value.
    pub const DEFAULT_FIELD: &str = " ";
    /// Default delimiter between entries.
    pub const DEFAULT_ENTRY: &str = "\n";

    /// Create from a literal (already-unescaped) string.
    #[must_use]
    pub fn from_literal(s: &str) -> Self {
        Self(s.to_string())
    }

    /// Create from an escaped string.
    #[must_use]
    pub fn from_escaped(s: &str) -> Self {
        Self(Self::unescape(s))
    }

    /// Get the unescaped delimiter for use in output.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Get a quoted representation suitable for verbose output.
    /// This ensures all delimiters are visible, including spaces.
    #[must_use]
    pub fn display_quoted(&self) -> String {
        format!("{:?}", self.0)
    }

    /// Unescape common escape sequences in a string.
    fn unescape(s: &str) -> String {
        let mut result = String::with_capacity(s.len());
        let mut chars = s.chars();

        while let Some(ch) = chars.next() {
            result.push(if ch == '\\' {
                chars.next().map_or('\\', |escape_code| match escape_code {
                    '0' => '\0',
                    'n' => '\n',
                    'r' => '\r',
                    't' => '\t',
                    '\\' => '\\',
                    '"' => '"',
                    c => c,
                })
            } else {
                ch
            });
        }

        result
    }
}

impl Default for Delimiter {
    fn default() -> Self {
        Self::from_literal(Self::DEFAULT_FIELD)
    }
}

impl Display for Delimiter {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_quoted())
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

impl AsRef<str> for Delimiter {
    fn as_ref(&self) -> &str {
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
