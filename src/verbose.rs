use crate::output::Output;
use anyhow::Result;
use word_tally::WordTally;

pub struct Verbose<'a> {
    output: Output,
    tally: &'a WordTally,
    delimiter: &'a str,
    source: &'a str,
}

impl<'a> Verbose<'a> {
    /// Constructs a new `Verbose` logger with the given output.
    pub const fn new(
        output: Output,
        tally: &'a WordTally,
        delimiter: &'a str,
        source: &'a str,
    ) -> Self {
        Self {
            output,
            tally,
            delimiter,
            source,
        }
    }

    /// Log verbose details.
    pub fn log(&mut self) -> Result<()> {
        self.log_details()?;
        self.log_options()?;
        self.log_filters()?;
        self.log_newline_with_entries()?;

        Ok(())
    }

    /// Log word tally details.
    fn log_details(&mut self) -> Result<()> {
        self.write_entry("source", self.source)?;
        self.write_entry("total-words", self.tally.count())?;
        self.write_entry("unique-words", self.tally.uniq_count())?;
        self.write_entry("delimiter", format!("{:?}", self.delimiter))?;

        Ok(())
    }

    /// Log word tally options.
    fn log_options(&mut self) -> Result<()> {
        self.write_entry("case", self.tally.options().case)?;
        self.write_entry("order", self.tally.options().sort)?;

        Ok(())
    }

    /// Log word tally filters.
    fn log_filters(&mut self) -> Result<()> {
        self.write_entry("min-chars", self.format(self.tally.filters().min_chars))?;
        self.write_entry("min-count", self.format(self.tally.filters().min_count))?;
        self.write_entry(
            "exclude-words",
            self.format(self.tally.filters().exclude.clone()),
        )?;

        Ok(())
    }

    /// Log a newline separator if the tally has entries.
    fn log_newline_with_entries(&mut self) -> Result<()> {
        if self.tally.count() > 0 {
            self.output.write_line("\n")?;
        }

        Ok(())
    }

    /// Format the `usize`, or `"none"` if none, as a `String`.
    fn format<T: ToString>(&self, value: Option<T>) -> String {
        value.map_or_else(|| "none".to_string(), |v| v.to_string())
    }

    /// Write a formatted log entry line.
    fn write_entry(&mut self, label: &str, value: impl ToString) -> Result<()> {
        self.output
            .write_line(&format!("{label}{}{}\n", self.delimiter, value.to_string()))
    }
}
