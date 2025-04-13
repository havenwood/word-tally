use crate::WordTally;
use clap::ValueEnum;
use core::cmp::Reverse;
use core::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

/// Tallying options.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Options {
    pub case: Case,
    pub sort: Sort,
}

/// Construct `Options`.
impl Options {
    pub const fn new(case: Case, sort: Sort) -> Self {
        Self { case, sort }
    }

    /// Create a new Options with default case and custom sort
    pub fn with_sort(sort: Sort) -> Self {
        Self { sort, ..Default::default() }
    }

    /// Create a new Options with custom case and default sort
    pub fn with_case(case: Case) -> Self {
        Self { case, ..Default::default() }
    }
}

/// Word case normalization options.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, ValueEnum, Serialize, Deserialize)]
pub enum Case {
    Original,
    Upper,
    #[default]
    Lower,
}

impl Case {
    /// Normalizes word case if a `Case` other than `Case::Original` is provided.
    pub fn normalize(&self, word: &str) -> Box<str> {
        match self {
            Self::Lower => word.to_lowercase().into_boxed_str(),
            Self::Upper => word.to_uppercase().into_boxed_str(),
            Self::Original => Box::from(word),
        }
    }
}

impl Display for Case {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let case = match self {
            Self::Lower => "lower",
            Self::Upper => "upper",
            Self::Original => "original",
        };

        f.write_str(case)
    }
}

/// Sort order by count.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, ValueEnum, Serialize, Deserialize)]
pub enum Sort {
    #[default]
    Desc,
    Asc,
    Unsorted,
}

impl Sort {
    /// Sorts the `tally` field in place if a sort order other than `Unsorted` is provided.
    pub fn apply(&self, w: &mut WordTally) {
        match self {
            Self::Desc => w.tally.sort_unstable_by_key(|&(_, count)| Reverse(count)),
            Self::Asc => w.tally.sort_unstable_by_key(|&(_, count)| count),
            Self::Unsorted => (),
        }
    }
}

impl Display for Sort {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let order = match self {
            Self::Desc => "desc",
            Self::Asc => "asc",
            Self::Unsorted => "unsorted",
        };

        f.write_str(order)
    }
}
