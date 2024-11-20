use crate::output::Output;
use anyhow::Result;
use word_tally::WordTally;

pub struct Verbose;

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
        self.write_entry(stderr, "case", delimiter, word_tally.options().case)?;
        self.write_entry(stderr, "order", delimiter, word_tally.options().sort)?;

        let filters = word_tally.filters();
        self.write_entry(
            stderr,
            "min-chars",
            delimiter,
            self.format(filters.min_chars),
        )?;
        self.write_entry(
            stderr,
            "min-count",
            delimiter,
            self.format(filters.min_count),
        )?;
        self.write_entry(
            stderr,
            "exclude-words",
            delimiter,
            self.format(filters.exclude.clone()),
        )?;

        if word_tally.count() > 0 {
            stderr.write_line("\n")?;
        }

        Ok(())
    }

    /// Format the `usize`, or `"none"` if none, as a `String`.
    fn format<T: ToString>(&self, value: Option<T>) -> String {
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
