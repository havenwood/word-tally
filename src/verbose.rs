//! Verbose logging functionality for word tallying operations.

use crate::output::Output;
use anyhow::{Context, Result};
use serde::Serialize;
use std::fmt::{Debug, Display};
use word_tally::{Format, WordTally};

/// Helper trait for formatting optional values.
trait FormatOption {
    fn format_option(self) -> String;
}

impl<T: Display> FormatOption for Option<T> {
    fn format_option(self) -> String {
        self.map_or_else(|| "none".to_string(), |v| v.to_string())
    }
}

impl<T: Display> FormatOption for &Option<T> {
    fn format_option(self) -> String {
        self.as_ref()
            .map_or_else(|| "none".to_string(), |v| v.to_string())
    }
}

/// Handles verbose output formatting and display of word tally results.
#[derive(Debug)]
pub struct Verbose {
    output: Output,
}

impl Default for Verbose {
    /// Default verbose logger writes to stderr.
    fn default() -> Self {
        Self {
            output: Output::stderr(),
        }
    }
}

/// Verbose information to be serialized.
#[derive(Debug)]
struct VerboseInfo<'verbose, 'tally> {
    tally: &'verbose WordTally<'tally>,
    source: &'verbose str,
}

impl<'verbose, 'tally> VerboseInfo<'verbose, 'tally> {
    /// Creates a new `VerboseInfo` instance.
    const fn new(tally: &'verbose WordTally<'tally>, source: &'verbose str) -> Self {
        Self { tally, source }
    }

    /// Convert an optional value to `Some(string)` or `None` for JSON serialization.
    fn value_or_null<T: Display>(option: &Option<T>) -> Option<String> {
        option.as_ref().map(|v| v.to_string())
    }
}

impl<'verbose, 'tally> Display for VerboseInfo<'verbose, 'tally> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let delimiter = self.tally.options().serialization().delimiter();
        let options = self.tally.options();
        let filters = options.filters();

        // Helper macro to write fields
        macro_rules! write_field {
            ($name:expr, $value:expr) => {
                writeln!(f, "{}{}{}", $name, delimiter, $value)?
            };
        }

        // Core metrics
        write_field!("source", self.source);
        write_field!("total-words", self.tally.count());
        write_field!("unique-words", self.tally.uniq_count());
        write_field!("delimiter", format!("{:?}", delimiter));

        // Options
        write_field!("case", options.case());
        write_field!("order", options.sort());
        write_field!("processing", options.processing());
        write_field!("io", options.io());

        // Filters
        write_field!("min-chars", filters.min_chars().format_option());
        write_field!("min-count", filters.min_count().format_option());
        write_field!("exclude-words", filters.exclude_words().format_option());
        write_field!(
            "exclude-patterns",
            filters.exclude_patterns().format_option()
        );
        write_field!(
            "include-patterns",
            filters.include_patterns().format_option()
        );

        Ok(())
    }
}

impl<'verbose, 'tally> Serialize for VerboseInfo<'verbose, 'tally> {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;

        let options = self.tally.options();
        let filters = options.filters();

        let mut s = serializer.serialize_struct("Verbose", 15)?;

        // Core metrics
        s.serialize_field("source", &self.source)?;
        s.serialize_field("totalWords", &self.tally.count())?;
        s.serialize_field("uniqueWords", &self.tally.uniq_count())?;

        // Options
        s.serialize_field("case", &options.case().to_string())?;
        s.serialize_field("order", &options.sort().to_string())?;
        s.serialize_field("processing", &options.processing().to_string())?;
        s.serialize_field("io", &options.io().to_string())?;

        // Filters
        s.serialize_field("minChars", &Self::value_or_null(&filters.min_chars()))?;
        s.serialize_field("minCount", &Self::value_or_null(&filters.min_count()))?;
        s.serialize_field(
            "excludeWords",
            &Self::value_or_null(filters.exclude_words()),
        )?;
        s.serialize_field(
            "excludePatterns",
            &Self::value_or_null(filters.exclude_patterns()),
        )?;
        s.serialize_field(
            "includePatterns",
            &Self::value_or_null(filters.include_patterns()),
        )?;

        s.end()
    }
}

impl Verbose {
    /// Formats and writes field-value pairs to output.
    fn write_fields(&mut self, info: &VerboseInfo<'_, '_>) -> Result<()> {
        let output = info.to_string();
        self.output
            .write_line(&output)
            .context("failed to write verbose output")?;

        Ok(())
    }

    /// Creates and writes a CSV with metrics and values.
    ///
    /// - metric: The name of the setting or statistic
    /// - value: The corresponding value
    fn build_csv_data(info: &VerboseInfo<'_, '_>) -> Result<String> {
        let mut wtr = csv::Writer::from_writer(Vec::new());
        wtr.write_record(["metric", "value"])
            .context("failed to write CSV header")?;

        // Get the verbose text output and parse it into CSV rows
        let text_output = info.to_string();
        let delimiter = info.tally.options().serialization().delimiter();

        for line in text_output.lines() {
            if let Some(separator_pos) = line.find(delimiter) {
                let metric = &line[..separator_pos];
                let value = &line[separator_pos + delimiter.len()..];
                wtr.write_record([metric, value])
                    .with_context(|| format!("failed to write CSV record for '{metric}'"))?;
            }
        }

        let buffer = wtr.into_inner().context("failed to extract CSV data")?;
        String::from_utf8(buffer).context("failed to convert CSV data to UTF-8")
    }

    /// Log verbose details in text format.
    fn log_text(&mut self, info: &VerboseInfo<'_, '_>) -> Result<()> {
        self.write_fields(info)?;

        // Add a newline separator if the tally has entries
        if info.tally.count() > 0 {
            self.output
                .write_line("\n")
                .context("failed to write newline separator")?;
        }

        Ok(())
    }

    /// Log verbose details in CSV format.
    fn log_csv(&mut self, info: &VerboseInfo<'_, '_>) -> Result<()> {
        let csv_data = Self::build_csv_data(info)?;
        self.output
            .write_line(&csv_data)
            .context("failed to write CSV data")?;
        self.output
            .write_line("\n")
            .context("failed to write trailing newline")?;

        Ok(())
    }

    /// Log verbose details in JSON format.
    fn log_json(&mut self, info: &VerboseInfo<'_, '_>) -> Result<()> {
        let json =
            serde_json::to_string(info).context("failed to serialize verbose info to JSON")?;
        self.output
            .write_line(&format!("{json}\n"))
            .context("failed to write JSON output")?;

        Ok(())
    }

    /// Writes verbose information for the word tally.
    ///
    /// Outputs information about the word tally in the specified format
    /// (JSON, CSV, or plain text).
    pub fn write_verbose_info<'tally>(
        &mut self,
        word_tally: &WordTally<'tally>,
        source: &str,
    ) -> Result<()> {
        let info = VerboseInfo::new(word_tally, source);

        match word_tally.options().serialization().format() {
            Format::Json => self.log_json(&info)?,
            Format::Csv => self.log_csv(&info)?,
            Format::Text => self.log_text(&info)?,
        }

        Ok(())
    }
}
