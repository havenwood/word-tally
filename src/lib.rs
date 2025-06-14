//! A tally of words with a count of the number of times each appears.
//!
//! `WordTally` tallies the number of times words appear in source input. Parallel streaming
//! is the default mode for balanced performance and memory usage. Memory-mapped I/O provides
//! the fastest processing for large files but requires seekable file descriptors. Sequential
//! streaming mode minimizes memory usage for constrained environments. All modes support both
//! files and stdin except memory-mapped which requires files.
//!
//! Word boundaries are determined using the [`icu_segmenter`](https://docs.rs/icu_segmenter/)
//! crate from ICU4X, which provides Unicode text segmentation following
//! the [Unicode Standard Annex #29](https://unicode.org/reports/tr29/) specification. The [`memchr`](https://docs.rs/memchr/)
//! crate provides SIMD-accelerated newline detection for efficient parallel chunk processing.
//!
//! # Configuration
//!
//! The [`Options`] struct provides a unified interface for configuring all aspects of word
//! tallying. See the [`options`] module for detailed configuration documentation.
//!
//! # Examples
//!
//! ```
//! use anyhow::Result;
//! use word_tally::{Options, TallyMap, WordTally};
//!
//! # fn example() -> Result<()> {
//! let options = Options::default();
//! let tally_map = TallyMap::from_buffered_input("example.txt", &options)?;
//! let words = WordTally::from_tally_map(tally_map, &options);
//! println!("Total words: {}", words.count());
//! # Ok(())
//! # }
//! ```
//!
//! ```
//! use anyhow::Result;
//! use word_tally::{Case, Io, Options, TallyMap, WordTally};
//!
//! # fn example() -> Result<()> {
//! // Memory-mapped file with lowercase normalization
//! let options = Options::default()
//!     .with_io(Io::ParallelMmap)
//!     .with_case(Case::Lower);
//! let tally_map = TallyMap::from_mapped_input("large-file.txt", &options)?;
//! let words = WordTally::from_tally_map(tally_map, &options);
//! # Ok(())
//! # }
//! ```

use std::{collections::HashMap, hash::BuildHasher, ops::Deref, path::Path, slice, str};

use serde::{Deserialize, Serialize};

mod error;
pub mod exit_code;
pub mod input;
pub mod options;
pub mod output;
pub mod tally_map;

pub use error::Error as WordTallyError;
pub use input::{Buffered, Mapped};
pub use options::{
    Options,
    case::Case,
    filters::{ExcludeWords, Filters, MinChars, MinCount},
    io::Io,
    patterns::{ExcludeSet, IncludeSet, PatternList},
    performance::Performance,
    serialization::Serialization,
    sort::Sort,
    threads::Threads,
};
pub use output::Output;
pub use tally_map::TallyMap;

/// The count of occurrences for a word.
pub type Count = usize;
/// A word represented as a boxed string.
pub type Word = Box<str>;
/// A collection of word-count pairs.
pub type Tally = Box<[(Word, Count)]>;

/// Provides metadata for data sources.
pub trait Metadata {
    /// Returns the file path, if file-based.
    fn path(&self) -> Option<&Path>;

    /// Returns the size in bytes, if known.
    fn size(&self) -> Option<u64>;
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
/// A tally of word frequencies and counts, along with processing options.
///
/// This type implements `Deref<Target = [(Word, Count)]>`, so you can directly access slice
/// methods like `len()`, `is_empty()`, `iter()`, indexing and such.
pub struct WordTally {
    /// Ordered pairs of words and the count of times they appear.
    tally: Tally,

    /// All of the options specified for how to tally.
    options: Options,

    /// The sum of all words tallied.
    count: Count,

    /// The sum of unique words tallied.
    uniq_count: Count,
}

impl WordTally {
    /// Creates a `WordTally` instance from a `TallyMap` and `Options`.
    #[must_use]
    pub fn from_tally_map(mut tally_map: TallyMap, options: &Options) -> Self {
        options.filters().apply(&mut tally_map, options.case());

        let count = tally_map.values().sum();
        let tally: Tally = tally_map.into_iter().collect();
        let uniq_count = tally.len();

        let mut instance = Self {
            options: options.clone(),
            tally,
            count,
            uniq_count,
        };

        instance.sort();

        instance
    }

    /// Gets the `tally` field.
    #[must_use]
    pub fn tally(&self) -> &[(Word, Count)] {
        &self.tally
    }

    /// Consumes the `tally` field.
    #[must_use]
    pub fn into_tally(self) -> Tally {
        self.tally
    }

    /// Gets a reference to the `options`.
    #[must_use]
    pub const fn options(&self) -> &Options {
        &self.options
    }

    /// Gets the `uniq_count` field.
    #[must_use]
    pub const fn uniq_count(&self) -> Count {
        self.uniq_count
    }

    /// Gets the `count` field.
    #[must_use]
    pub const fn count(&self) -> Count {
        self.count
    }

    /// Sorts the `tally` field in place if a sort order other than `Unsorted` is provided.
    fn sort(&mut self) {
        self.options
            .sort()
            .apply(&mut self.tally, self.options.io());
    }
}

impl Deref for WordTally {
    type Target = [(Word, Count)];

    fn deref(&self) -> &Self::Target {
        &self.tally
    }
}

/// Converts a `WordTally` into a `Vec<(Word, Count)>`.
impl From<WordTally> for Vec<(Word, Count)> {
    fn from(word_tally: WordTally) -> Self {
        word_tally.into_tally().into_vec()
    }
}

/// Converts a `WordTally` into a `HashMap<Word, Count>`.
impl<S: BuildHasher + Default> From<WordTally> for HashMap<Word, Count, S> {
    fn from(word_tally: WordTally) -> Self {
        word_tally.into_tally().into_iter().collect()
    }
}

/// Allows consuming the `WordTally` in a for loop, yielding owned pairs.
impl IntoIterator for WordTally {
    type Item = (Word, Count);
    type IntoIter = <Box<[(Word, Count)]> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.tally.into_iter()
    }
}

/// Makes `WordTally` reference available directly in a for loop.
impl<'i> IntoIterator for &'i WordTally {
    type Item = &'i (Word, Count);
    type IntoIter = slice::Iter<'i, (Word, Count)>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
