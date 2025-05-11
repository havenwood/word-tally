//! Word case normalization options.

use crate::Word;
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
    /// Normalizes word case if a `Case` other than `Case::Original` is provided.
    ///
    /// # Examples
    ///
    /// ```
    /// use word_tally::Case;
    ///
    /// let hello = "Hello";
    /// assert_eq!(Case::Lower.normalize(hello), "hello".into());
    /// assert_eq!(Case::Upper.normalize(hello), "HELLO".into());
    /// assert_eq!(Case::Original.normalize(hello), hello.into());
    /// ```
    pub fn normalize(&self, word: &str) -> Word {
        match self {
            Self::Lower => word.to_lowercase().into_boxed_str(),
            Self::Upper => word.to_uppercase().into_boxed_str(),
            Self::Original => Box::from(word),
        }
    }
}
