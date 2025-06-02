//! Sorting options for word tallying results.

use crate::{Count, Io, Word};
use clap::ValueEnum;
use core::cmp::Reverse;
use core::fmt::{self, Display, Formatter};
use rayon::slice::ParallelSliceMut;
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
#[serde(rename_all = "camelCase")]
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

    /// Sorts the tally in place based on the sort order and I/O mode.
    pub fn apply(&self, tally: &mut [(Word, Count)], io: Io) {
        match (self, io) {
            // No sorting
            (Self::Unsorted, _) => {}

            // Sequential unstable sorting
            (Self::Desc, Io::Stream) => {
                tally.sort_unstable_by_key(|&(_, count)| Reverse(count));
            }
            (Self::Asc, Io::Stream) => {
                tally.sort_unstable_by_key(|&(_, count)| count);
            }

            // Parallel unstable sorting
            (
                Self::Desc,
                Io::ParallelStream | Io::ParallelInMemory | Io::ParallelMmap | Io::ParallelBytes,
            ) => {
                tally.par_sort_unstable_by_key(|&(_, count)| Reverse(count));
            }
            (
                Self::Asc,
                Io::ParallelStream | Io::ParallelInMemory | Io::ParallelMmap | Io::ParallelBytes,
            ) => {
                tally.par_sort_unstable_by_key(|&(_, count)| count);
            }
        }
    }
}
