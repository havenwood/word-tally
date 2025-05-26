//! Word case normalization options.

use clap::ValueEnum;
use core::fmt::{self, Display, Formatter};
use serde::{Deserialize, Serialize};

/// Word case normalization options.
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
pub enum Case {
    Original,
    Upper,
    #[default]
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
    /// Create a string from the uppercase, lowercase or original.
    #[must_use]
    pub fn normalize(&self, content: &str) -> String {
        match self {
            Self::Lower => content.to_lowercase(),
            Self::Upper => content.to_uppercase(),
            Self::Original => content.to_string(),
        }
    }
}
