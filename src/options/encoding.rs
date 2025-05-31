//! Word encoding strategies for boundary detection.

use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};

/// Determines how word boundaries are detected.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, ValueEnum,
)]
#[serde(rename_all = "camelCase")]
#[value(rename_all = "lower")]
pub enum Encoding {
    /// Unicode-compliant word segmentation (default).
    #[value(alias = "utf8", alias = "utf-8")]
    Unicode,

    /// ASCII-only word segmentation using whitespace and punctuation.
    Ascii,
}

impl Default for Encoding {
    fn default() -> Self {
        Self::Unicode
    }
}

impl Display for Encoding {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unicode => write!(f, "unicode"),
            Self::Ascii => write!(f, "ascii"),
        }
    }
}

impl Encoding {
    /// Whether this encoding uses Unicode word boundaries.
    #[must_use]
    pub const fn is_unicode(&self) -> bool {
        matches!(self, Self::Unicode)
    }

    /// Whether this encoding uses ASCII-only word boundaries.
    #[must_use]
    pub const fn is_ascii(&self) -> bool {
        matches!(self, Self::Ascii)
    }
}
