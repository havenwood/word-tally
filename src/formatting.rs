use crate::{Processing, Word, WordTally};
use clap::ValueEnum;
use core::cmp::Reverse;
use core::fmt::{self, Display, Formatter};
use rayon::slice::ParallelSliceMut;

use serde::{Deserialize, Serialize};

/// Output format options.
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
pub enum Format {
    #[default]
    Text,
    Json,
    Csv,
}

impl Display for Format {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Text => write!(f, "text"),
            Self::Json => write!(f, "json"),
            Self::Csv => write!(f, "csv"),
        }
    }
}

/// Formatting options for word tallying.
#[derive(
    Clone, Copy, Debug, Default, Eq, PartialEq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct Formatting {
    case: Case,
    sort: Sort,
    format: Format,
}

impl fmt::Display for Formatting {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Formatting {{ case: {}, sort: {}, format: {} }}",
            self.case, self.sort, self.format
        )
    }
}

/// Construct `Formatting`.
impl Formatting {
    pub const fn new(case: Case, sort: Sort, format: Format) -> Self {
        Self { case, sort, format }
    }

    /// With custom sort
    pub fn with_sort(sort: Sort) -> Self {
        Self {
            sort,
            ..Default::default()
        }
    }

    /// With custom case
    pub fn with_case(case: Case) -> Self {
        Self {
            case,
            ..Default::default()
        }
    }

    /// With custom format
    pub fn with_format(format: Format) -> Self {
        Self {
            format,
            ..Default::default()
        }
    }

    /// Set the case option and return a new instance
    pub const fn with_case_setting(mut self, case: Case) -> Self {
        self.case = case;
        self
    }

    /// Set the sort option and return a new instance
    pub const fn with_sort_setting(mut self, sort: Sort) -> Self {
        self.sort = sort;
        self
    }

    /// Set the format option and return a new instance
    pub const fn with_format_setting(mut self, format: Format) -> Self {
        self.format = format;
        self
    }

    /// Get the case setting
    pub const fn case(&self) -> Case {
        self.case
    }

    /// Get the sort setting
    pub const fn sort(&self) -> Sort {
        self.sort
    }

    /// Get the format setting
    pub const fn format(&self) -> Format {
        self.format
    }
}

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
    pub fn normalize(&self, word: &str) -> Word {
        match self {
            Self::Lower => word.to_lowercase().into_boxed_str(),
            Self::Upper => word.to_uppercase().into_boxed_str(),
            Self::Original => Box::from(word),
        }
    }
}

/// Sort order by count.
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
