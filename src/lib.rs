use anyhow::Result;
use clap_stdin::FileOrStdin;
use core::cmp::Reverse;
use std::collections::HashMap;
use std::io::{self, BufRead, BufReader, Lines, Read};
use std::path::PathBuf;
use std::process::exit;
use unicode_segmentation::UnicodeSegmentation;

/// A `WordTally` represents an ordered tally of words paired with their count.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct WordTally {
    /// Whether the tally field has been sorted by the `sort` method.
    sorted: bool,
    /// Ordered pairs of words and the count of times they appear.
    tally: Vec<(String, u32)>,
}

impl WordTally {
    /// Constructs a new `WordTally` from a file or stdin source input.
    #[must_use]
    pub fn new(input: &FileOrStdin<PathBuf>, case_sensitive: bool, sort: bool) -> Self {
        let mut word_tally = Self {
            sorted: false,
            tally: Vec::from_iter(Self::tally_words(Self::lines(input), case_sensitive)),
        };

        if sort {
            word_tally.sort();
        }

        word_tally
    }

    /// Sorts the tally field in place or does nothing if already sorted.
    pub fn sort(&mut self) {
        if self.sorted {
            return;
        }

        self.tally
            .sort_unstable_by_key(|&(_, count)| Reverse(count));
        self.sorted = true;
    }

    /// Getter for `sorted` field
    pub fn sorted(&self) -> bool {
        self.sorted
    }

    /// Getter for `tally` field
    pub fn tally(&self) -> &Vec<(String, u32)> {
        &self.tally
    }

    /// Count the sum of all unique words in the tally.
    pub fn uniq_count(&self) -> Result<u32> {
        Ok(u32::try_from(self.tally.len())?)
    }

    /// Count the total sum of all words in the tally.
    pub fn count(&self) -> u32 {
        self.tally.iter().map(|&(_, count)| count).sum()
    }

    /// Find the mean average word count if there are words.
    pub fn avg(&self) -> Option<f32> {
        let total = self.count();

        if total > 0 {
            let uniq = self.uniq_count().ok()?;
            let avg = f64::from(total) / f64::from(uniq);

            Some(avg as f32)
        } else {
            None
        }
    }

    /// Creates a line buffer reader from a file or stdin source.
    fn lines(input: &FileOrStdin<PathBuf>) -> Lines<BufReader<impl Read>> {
        match input.into_reader() {
            Ok(readable) => io::BufReader::new(readable).lines(),
            Err(err) => {
                eprintln!("{err} -- {:#?}", input.source);
                exit(1);
            }
        }
    }

    /// Creates a tally of words from a line buffer reader.
    fn tally_words(
        lines: io::Lines<BufReader<impl Read>>,
        case_sensitive: bool,
    ) -> HashMap<String, u32> {
        let mut tally = HashMap::new();

        for line in lines.map_while(Result::ok) {
            line.unicode_words().for_each(|unicode_word| {
                let word = if case_sensitive {
                    unicode_word.to_owned()
                } else {
                    unicode_word.to_lowercase()
                };

                tally
                    .entry(word)
                    .and_modify(|count| *count += 1)
                    .or_insert(1);
            });
        }

        tally
    }
}
