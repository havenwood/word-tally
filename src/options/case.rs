//! Word case normalization options.

use core::fmt::{self, Display, Formatter};
use std::borrow::Cow;

use clap::ValueEnum;
use serde::{Deserialize, Serialize};

/// Word case normalization options.
///
/// # Examples
///
/// ```
/// use word_tally::Case;
///
/// // "Hello" and "hello" counted separately
/// let original = Case::Original;
///
/// // "Hello" and "hello" both become "hello"
/// let lower = Case::Lower;
///
/// // "Hello" and "hello" both become "HELLO"
/// let upper = Case::Upper;
/// ```
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    ValueEnum,
)]
#[serde(rename_all = "camelCase")]
pub enum Case {
    /// Keep original case.
    #[default]
    Original,
    /// Convert to uppercase.
    Upper,
    /// Convert to lowercase.
    Lower,
}

impl Display for Case {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Lower => write!(f, "lower"),
            Self::Upper => write!(f, "upper"),
            Self::Original => write!(f, "original"),
        }
    }
}

impl Case {
    /// Normalize text using Unicode case conversion, returning a reference if already normalized or
    /// owned if conversion needed.
    #[must_use]
    pub fn normalize_unicode<'a>(&self, content: &'a str) -> Cow<'a, str> {
        match self {
            Self::Lower => {
                if content.chars().all(|c| !c.is_uppercase()) {
                    Cow::Borrowed(content)
                } else {
                    Cow::Owned(content.to_lowercase())
                }
            }
            Self::Upper => {
                if content.chars().all(|c| !c.is_lowercase()) {
                    Cow::Borrowed(content)
                } else {
                    Cow::Owned(content.to_uppercase())
                }
            }
            Self::Original => Cow::Borrowed(content),
        }
    }

    /// ASCII-specific normalization for better performance, returning a reference if already
    /// normalized or owned if conversion needed.
    #[must_use]
    pub fn normalize_ascii<'a>(&self, content: &'a str) -> Cow<'a, str> {
        match self {
            Self::Lower => {
                if content.bytes().all(|b| !b.is_ascii_uppercase()) {
                    Cow::Borrowed(content)
                } else {
                    Cow::Owned(content.to_ascii_lowercase())
                }
            }
            Self::Upper => {
                if content.bytes().all(|b| !b.is_ascii_lowercase()) {
                    Cow::Borrowed(content)
                } else {
                    Cow::Owned(content.to_ascii_uppercase())
                }
            }
            Self::Original => Cow::Borrowed(content),
        }
    }
}
