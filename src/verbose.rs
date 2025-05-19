//! Verbose logging functionality for word tallying operations.

use crate::output::Output;
use anyhow::{Context, Result};
use serde::Serialize;
use word_tally::{Format, WordTally};

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

/// Verbose data that can be serialized to both JSON and CSV.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct VerboseData<'a> {
    source: &'a str,
    total_words: usize,
    unique_words: usize,
    delimiter: String,
    case: String,
    order: String,
    processing: String,
    io: String,
    min_chars: Option<usize>,
    min_count: Option<usize>,
    exclude_words: &'a Option<word_tally::ExcludeWords>,
    exclude_patterns: &'a Option<word_tally::ExcludePatterns>,
    include_patterns: &'a Option<word_tally::IncludePatterns>,
}

impl<'a> VerboseData<'a> {
    /// Create from WordTally and source.
    fn from_tally(tally: &'a WordTally<'a>, source: &'a str) -> Self {
        let options = tally.options();
        let filters = options.filters();
        let serialization = options.serialization();

        Self {
            source,
            total_words: tally.count(),
            unique_words: tally.uniq_count(),
            delimiter: format!("{:?}", serialization.delimiter()),
            case: options.case().to_string(),
            order: options.sort().to_string(),
            processing: options.processing().to_string(),
            io: options.io().to_string(),
            min_chars: filters.min_chars(),
            min_count: filters.min_count(),
            exclude_words: filters.exclude_words(),
            exclude_patterns: filters.exclude_patterns(),
            include_patterns: filters.include_patterns(),
        }
    }

    /// Get all fields as name-value pairs.
    fn field_pairs(&self) -> Vec<(&str, String)> {
        let mut pairs = Vec::with_capacity(13);
        pairs.extend_from_slice(&[
            ("source", self.source.to_string()),
            ("total-words", self.total_words.to_string()),
            ("unique-words", self.unique_words.to_string()),
            ("delimiter", self.delimiter.clone()),
            ("case", self.case.clone()),
            ("order", self.order.clone()),
            ("processing", self.processing.clone()),
            ("io", self.io.clone()),
            (
                "min-chars",
                self.min_chars.map_or("none".to_string(), |v| v.to_string()),
            ),
            (
                "min-count",
                self.min_count.map_or("none".to_string(), |v| v.to_string()),
            ),
            (
                "exclude-words",
                self.exclude_words
                    .as_ref()
                    .map_or("none".to_string(), |v| v.to_string()),
            ),
            (
                "exclude-patterns",
                self.exclude_patterns
                    .as_ref()
                    .map_or("none".to_string(), |v| v.to_string()),
            ),
            (
                "include-patterns",
                self.include_patterns
                    .as_ref()
                    .map_or("none".to_string(), |v| v.to_string()),
            ),
        ]);
        pairs
    }
}

impl Verbose {
    /// Writes verbose information for the word tally.
    pub fn write_verbose_info(&mut self, word_tally: &WordTally<'_>, source: &str) -> Result<()> {
        let data = VerboseData::from_tally(word_tally, source);

        match word_tally.options().serialization().format() {
            Format::Json => self.write_json(&data),
            Format::Csv => self.write_csv(&data),
            Format::Text => {
                self.write_text(&data, word_tally.options().serialization().delimiter())
            }
        }
    }

    /// Write verbose info in JSON format.
    fn write_json(&mut self, data: &VerboseData<'_>) -> Result<()> {
        let json =
            serde_json::to_string(data).context("failed to serialize verbose info to JSON")?;

        self.output
            .write_chunk(&format!("{}\n\n", json))
            .context("failed to write JSON output")
    }

    /// Write verbose info in CSV format.
    fn write_csv(&mut self, data: &VerboseData<'_>) -> Result<()> {
        let mut writer = csv::Writer::from_writer(Vec::new());
        let field_pairs = data.field_pairs();

        // Write headers directly from iterator
        writer.write_record(field_pairs.iter().map(|(name, _)| *name))?;

        // Write data values directly from iterator
        writer.write_record(field_pairs.iter().map(|(_, value)| value))?;

        self.format_and_write_output(writer.into_inner()?)
    }

    /// Write verbose info in text format.
    fn write_text(&mut self, data: &VerboseData<'_>, delimiter: &str) -> Result<()> {
        // Write each field as key-value pairs
        for (field_name, value) in data.field_pairs() {
            self.output
                .write_chunk(&format!("{}{}{}\n", field_name, delimiter, value))?;
        }

        // Add separator if needed
        if data.total_words > 0 {
            self.output.write_chunk("\n")?;
        }

        Ok(())
    }

    /// Helper to format and write output.
    fn format_and_write_output(&mut self, data: Vec<u8>) -> Result<()> {
        let output = String::from_utf8(data).context("failed to convert output to UTF-8")?;

        self.output
            .write_chunk(&output)
            .context("failed to write output")?;
        self.output
            .write_chunk("\n")
            .context("failed to write trailing newline")
    }
}
