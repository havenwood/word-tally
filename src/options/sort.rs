//! Sorting options for word tallying results.

use crate::WordTally;
use crate::options::processing::Processing;
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
    /// Unstable sort with `Rayon` when `Parallel` processing is enabled.
    pub fn apply(&self, w: &mut WordTally<'_>) {
        match (self, w.options().processing()) {
            // No sorting
            (Self::Unsorted, _) => {}

            // Parallel unstable sorting
            (Self::Desc, Processing::Parallel) => w
                .tally
                .par_sort_unstable_by_key(|&(_, count)| Reverse(count)),
            (Self::Asc, Processing::Parallel) => {
                w.tally.par_sort_unstable_by_key(|&(_, count)| count)
            }

            // Sequential stable sorting
            (Self::Desc, Processing::Sequential) => {
                w.tally.sort_by_key(|&(_, count)| Reverse(count))
            }
            (Self::Asc, Processing::Sequential) => w.tally.sort_by_key(|&(_, count)| count),
        }
    }
}
