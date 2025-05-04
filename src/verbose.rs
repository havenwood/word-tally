use crate::formatting::Format;
use crate::output::Output;
use anyhow::{Context, Result};
use std::fmt::{self, Debug, Formatter};
use word_tally::WordTally;

pub struct Verbose<'a, 'b> {
    output: &'a mut Output,
    tally: &'a WordTally<'b>,
    delimiter: &'a str,
    source: &'a str,
}

impl Debug for Verbose<'_, '_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Verbose")
            .field("output", self.output)
            .field("tally", self.tally)
            .field("delimiter", &self.delimiter)
            .field("source", &self.source)
            .finish()
    }
}

impl<'a, 'b> Verbose<'a, 'b> {
    /// Constructs a new `Verbose` logger with the given output.
    pub const fn new(
        output: &'a mut Output,
        tally: &'a WordTally<'b>,
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

    /// Log verbose details in text format.
    pub fn log(&mut self) -> Result<()> {
        self.log_details()?;
        self.log_options()?;
        self.log_filters()?;
        self.log_newline_with_entries()?;

        Ok(())
    }

    /// Log verbose details in CSV format.
    pub fn log_csv(&mut self) -> Result<()> {
        let mut wtr = csv::Writer::from_writer(Vec::new());
        wtr.write_record(["metric", "value"])?;
        wtr.write_record(["source", self.source])?;
        wtr.write_record(["total-words", &self.tally.count().to_string()])?;
        wtr.write_record(["unique-words", &self.tally.uniq_count().to_string()])?;
        wtr.write_record(["delimiter", &format!("{:?}", self.delimiter)])?;
        wtr.write_record(["case", &self.tally.options().case().to_string()])?;
        wtr.write_record(["order", &self.tally.options().sort().to_string()])?;
        wtr.write_record(["min-chars", &self.format(*self.tally.filters().min_chars())])?;
        wtr.write_record(["min-count", &self.format(*self.tally.filters().min_count())])?;
        wtr.write_record([
            "exclude-words",
            &self.format(self.tally.filters().exclude_words().clone()),
        ])?;
        wtr.write_record([
            "exclude-patterns",
            &self.format(self.tally.filters().exclude_patterns().as_ref()),
        ])?;
        let csv_data = String::from_utf8(wtr.into_inner()?)?;
        self.output.write_line(&csv_data)?;

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
        self.write_entry("case", self.tally.options().case())?;
        self.write_entry("order", self.tally.options().sort())?;

        Ok(())
    }

    /// Log word tally filters.
    fn log_filters(&mut self) -> Result<()> {
        self.write_entry("min-chars", self.format(*self.tally.filters().min_chars()))?;
        self.write_entry("min-count", self.format(*self.tally.filters().min_count()))?;
        self.write_entry(
            "exclude-words",
            self.format(self.tally.filters().exclude_words().clone()),
        )?;
        self.write_entry(
            "exclude-patterns",
            self.format(self.tally.filters().exclude_patterns().as_ref()),
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
    pub fn format<T: ToString>(&self, value: Option<T>) -> String {
        value.map_or_else(|| "none".to_string(), |v| v.to_string())
    }

    /// Write a formatted log entry line.
    fn write_entry(&mut self, label: &str, value: impl ToString) -> Result<()> {
        self.output
            .write_line(&format!("{label}{}{}\n", self.delimiter, value.to_string()))
    }

    /// Create a JSON representation of the verbose output.
    pub fn to_json(&self) -> Result<String> {
        use serde_json::json;

        let value = json!({
            "source": self.source,
            "total-words": self.tally.count(),
            "unique-words": self.tally.uniq_count(),
            "delimiter": format!("{:?}", self.delimiter),
            "case": self.tally.options().case().to_string(),
            "order": self.tally.options().sort().to_string(),
            "min-chars": self.format(*self.tally.filters().min_chars()),
            "min-count": self.format(*self.tally.filters().min_count()),
            "exclude-words": self.format(self.tally.filters().exclude_words().clone()),
            "exclude-patterns": self.format(self.tally.filters().exclude_patterns().as_ref()),
        });

        serde_json::to_string(&value).with_context(|| "Failed to serialize verbose info to JSON")
    }
}

/// Output the selected format to stderr if verbose is enabled.
pub fn handle_output(
    is_verbose: bool,
    format: Format,
    word_tally: &WordTally<'_>,
    delimiter: &str,
    source: &str,
) -> Result<()> {
    if !is_verbose {
        return Ok(());
    }

    match format {
        Format::Json => output_json(word_tally, delimiter, source),
        Format::Csv => output_csv(word_tally, delimiter, source),
        Format::Text => output_text(word_tally, delimiter, source),
    }
}

/// Handle verbose output based on word tally results.
pub fn handle_verbose_output(
    is_verbose: bool,
    format: Format,
    word_tally: &WordTally<'_>,
    delimiter: &str,
    source: &str,
) -> Result<()> {
    handle_output(is_verbose, format, word_tally, delimiter, source)
}

fn output_json(word_tally: &WordTally<'_>, delimiter: &str, source: &str) -> Result<()> {
    let mut stderr = Output::stderr();
    let verbose = Verbose::new(&mut stderr, word_tally, delimiter, source);
    let json = verbose.to_json()?;
    stderr.write_line(&format!("{json}\n\n"))
}

fn output_csv(word_tally: &WordTally<'_>, delimiter: &str, source: &str) -> Result<()> {
    let mut stderr = Output::stderr();
    {
        let mut verbose = Verbose::new(&mut stderr, word_tally, delimiter, source);
        verbose.log_csv()?;
    }
    stderr.write_line("\n")
}

fn output_text(word_tally: &WordTally<'_>, delimiter: &str, source: &str) -> Result<()> {
    let mut stderr = Output::stderr();
    let mut verbose = Verbose::new(&mut stderr, word_tally, delimiter, source);
    verbose.log()
}
