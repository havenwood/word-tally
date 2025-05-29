//! Sorting options for word tallying results.

use crate::WordTally;
use crate::options::io::Io;
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
    Eq,
    PartialEq,
    PartialOrd,
    Ord,
    Hash,
    ValueEnum,
    Serialize,
    Deserialize,
)]
pub enum Sort {
    #[default]
    Desc,
    Asc,
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
    /// Sorts the `tally` field in place if a sort order other than `Unsorted` is provided.
    /// Uses unstable sort, with parallel sorting for parallel I/O modes.
    pub fn apply(&self, w: &mut WordTally<'_>) {
        match (self, w.options().io()) {
            // No sorting
            (Self::Unsorted, _) => {}

            // Sequential unstable sorting
            (Self::Desc, Io::Stream) => {
                w.tally.sort_unstable_by_key(|&(_, count)| Reverse(count));
            }
            (Self::Asc, Io::Stream) => {
                w.tally.sort_unstable_by_key(|&(_, count)| count);
            }

            // Parallel unstable sorting
            (
                Self::Desc,
                Io::ParallelStream | Io::ParallelInMemory | Io::ParallelMmap | Io::ParallelBytes,
            ) => w
                .tally
                .par_sort_unstable_by_key(|&(_, count)| Reverse(count)),
            (
                Self::Asc,
                Io::ParallelStream | Io::ParallelInMemory | Io::ParallelMmap | Io::ParallelBytes,
            ) => {
                w.tally.par_sort_unstable_by_key(|&(_, count)| count);
            }
        }
    }
}
