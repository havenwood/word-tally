use crate::output::Output;
use anyhow::Result;
use word_tally::{Case, Sort, WordTally};

/// `Verbose` contains some config details to be logged.
pub struct Verbose {
    pub case: Case,
    pub sort: Sort,
    pub min_chars: Option<usize>,
    pub min_count: Option<usize>,
}

impl Verbose {
    /// Log word tally details to stderr.
    pub fn log(
        &self,
        stderr: &mut Output,
        word_tally: &WordTally,
        delimiter: &str,
        source: &str,
    ) -> Result<()> {
        self.write_entry(stderr, "source", delimiter, source)?;
        self.write_entry(stderr, "total-words", delimiter, word_tally.count())?;
        self.write_entry(stderr, "unique-words", delimiter, word_tally.uniq_count())?;
        self.write_entry(stderr, "delimiter", delimiter, format!("{:?}", delimiter))?;
        self.write_entry(stderr, "case", delimiter, self.case)?;
        self.write_entry(stderr, "order", delimiter, self.sort)?;
        self.write_entry(stderr, "min-chars", delimiter, self.format(self.min_chars))?;
        self.write_entry(stderr, "min-count", delimiter, self.format(self.min_count))?;

        if word_tally.count() > 0 {
            stderr.write_line("\n")?;
        }

        Ok(())
    }

    /// Format `"none"` or a `usize` as a `String`.
    fn format(&self, value: Option<usize>) -> String {
        value.map_or_else(|| "none".to_string(), |v| v.to_string())
    }

    /// Write a formatted log entry line.
    fn write_entry(
        &self,
        w: &mut Output,
        label: &str,
        delimiter: &str,
        value: impl ToString,
    ) -> Result<()> {
        w.write_line(&format!("{label}{delimiter}{}\n", value.to_string()))
    }
}
