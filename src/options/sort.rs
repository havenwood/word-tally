//! Sorting options for word tallying results.

use clap::ValueEnum;
use core::fmt::{self, Display, Formatter};
use serde::{Deserialize, Serialize};

/// Sort order by count.
///
/// # Examples
///
/// ```
/// use word_tally::Sort;
///
/// assert_eq!(Sort::default(), Sort::Desc);
/// assert_eq!(Sort::Desc.to_string(), "desc");
/// assert_eq!(Sort::Asc.to_string(), "asc");
/// assert_eq!(Sort::Unsorted.to_string(), "unsorted");
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
pub enum Sort {
    /// Sort by count descending.
    #[default]
    Desc,
    /// Sort by count ascending.
    Asc,
    /// No sorting applied.
    Unsorted,
}

impl Display for Sort {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Desc => write!(f, "desc"),
            Self::Asc => write!(f, "asc"),
            Self::Unsorted => write!(f, "unsorted"),
        }
    }
}

impl Sort {
    /// Returns true if sorting should be applied.
    #[must_use]
    pub const fn should_sort(&self) -> bool {
        !matches!(self, Self::Unsorted)
    }

    /// Returns true if sorting should be in descending order.
    #[must_use]
    pub const fn is_descending(&self) -> bool {
        matches!(self, Self::Desc)
    }
}
