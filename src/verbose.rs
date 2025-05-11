//! Verbose logging functionality for word tallying operations.

use crate::output::Output;
use anyhow::{Context, Result};
use serde::Serialize;
use std::fmt::{Debug, Display};
use word_tally::serialization::Format;
use word_tally::{
    Case, Count, ExcludePatterns, ExcludeWords, IncludePatterns, Io, MinChars, MinCount,
    Processing, Sort, WordTally,
};

/// Module for serializing Option<T> types that implement Display
mod option_display_format {
    use serde::Serializer;
    use std::fmt::Display;

    /// Serializes an Option<T> as a string or null
    pub fn serialize<T, S>(value: &Option<T>, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: Display,
        S: Serializer,
    {
        match value {
            Some(v) => serializer.serialize_str(&v.to_string()),
            None => serializer.serialize_none(),
        }
    }
}

/// Module for serializing Display types as strings
mod display_format {
    use serde::Serializer;
    use std::fmt::Display;

    /// Serializes any type implementing Display as a string
    pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: Display,
        S: Serializer,
    {
        serializer.serialize_str(&value.to_string())
    }
}

/// Handles verbose output formatting and display of word tally results.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Verbose<'v, 'a> {
    // Skip fields that should not be serialized
    #[serde(skip)]
    output: &'v mut Output,
    #[serde(skip)]
    tally: &'v WordTally<'a>,
    #[serde(skip)]
    delimiter: &'v str,

    // Core metrics (use default serialization)
    source: &'v str,
    total_words: Count,
    unique_words: Count,

    // Options - serialize enums as strings
    #[serde(with = "display_format")]
    case: Case,
    #[serde(with = "display_format")]
    order: Sort,
    #[serde(with = "display_format")]
    processing: Processing,
    #[serde(with = "display_format")]
    io: Io,

    // Filters - serialize as strings or null
    #[serde(with = "option_display_format")]
    min_chars: Option<MinChars>,
    #[serde(with = "option_display_format")]
    min_count: Option<MinCount>,

    // References to complex types
    #[serde(with = "option_display_format")]
    exclude_words: Option<&'v ExcludeWords>,
    #[serde(with = "option_display_format")]
    exclude_patterns: Option<&'v ExcludePatterns>,
    #[serde(with = "option_display_format")]
    include_patterns: Option<&'v IncludePatterns>,
}

impl<'v, 'a> Verbose<'v, 'a> {
    /// Constructs a new `Verbose` logger with the given output.
    ///
    /// # Arguments
    ///
    /// * `output` - The output destination for verbose information
    /// * `tally` - The word tally containing results to display
    /// * `delimiter` - The delimiter to use between labels and values
    /// * `source` - The source of the text (filename or description)
    pub const fn new(
        output: &'v mut Output,
        tally: &'v WordTally<'a>,
        delimiter: &'v str,
        source: &'v str,
    ) -> Self {
        let options = tally.options();
        let filters = options.filters();

        Self {
            output,
            tally,
            delimiter,
            source,
            total_words: tally.count(),
            unique_words: tally.uniq_count(),
            case: options.case(),
            order: options.sort(),
            processing: options.processing(),
            io: options.io(),
            min_chars: filters.min_chars(),
            min_count: filters.min_count(),
            exclude_words: filters.exclude_words().as_ref(),
            exclude_patterns: filters.exclude_patterns().as_ref(),
            include_patterns: filters.include_patterns().as_ref(),
        }
    }

    /// Format option as display string
    fn format_option_str<T: Display>(value: Option<&T>) -> String {
        value.map_or_else(|| "none".to_string(), |v| v.to_string())
    }

    /// Formats and writes field-value pairs to output.
    fn write_fields(&mut self) -> Result<()> {
        // Pre-allocate a string buffer with sufficient capacity
        let mut output = String::with_capacity(512);

        // Helper to add a field-value line to the buffer
        let add_field = |output: &mut String, name: &str, value: &dyn Display| {
            use std::fmt::Write;
            let _ = writeln!(output, "{}{}{}", name, self.delimiter, value);
        };

        // Core metrics
        add_field(&mut output, "source", &self.source);
        add_field(&mut output, "total-words", &self.total_words);
        add_field(&mut output, "unique-words", &self.unique_words);
        add_field(&mut output, "delimiter", &format!("{:?}", self.delimiter));

        // Options
        add_field(&mut output, "case", &self.case);
        add_field(&mut output, "order", &self.order);
        add_field(&mut output, "processing", &self.processing);
        add_field(&mut output, "io", &self.io);

        // Filters - use the helper for Option<T> formatting
        add_field(
            &mut output,
            "min-chars",
            &Self::format_option_str(self.min_chars.as_ref()),
        );
        add_field(
            &mut output,
            "min-count",
            &Self::format_option_str(self.min_count.as_ref()),
        );
        add_field(
            &mut output,
            "exclude-words",
            &Self::format_option_str(self.exclude_words),
        );
        add_field(
            &mut output,
            "exclude-patterns",
            &Self::format_option_str(self.exclude_patterns),
        );
        add_field(
            &mut output,
            "include-patterns",
            &Self::format_option_str(self.include_patterns),
        );

        // Write all at once
        self.output.write_line(&output)?;
        Ok(())
    }

    /// Creates and writes a CSV with metrics and values.
    ///
    /// - metric: The name of the setting or statistic
    /// - value: The corresponding value
    fn build_csv_data(&self) -> Result<String> {
        let mut wtr = csv::Writer::from_writer(Vec::new());
        wtr.write_record(["metric", "value"])
            .with_context(|| "Failed to write CSV header")?;

        // Helper to write records with automatic conversion to string
        let mut write_record = |name: &str, value: &dyn Display| -> Result<()> {
            wtr.write_record([name, &value.to_string()])
                .with_context(|| format!("Failed to write CSV record for '{name}'"))
        };

        // Core metrics
        write_record("source", &self.source)?;
        write_record("total-words", &self.total_words)?;
        write_record("unique-words", &self.unique_words)?;
        write_record("delimiter", &format!("{:?}", self.delimiter))?;

        // Options
        write_record("case", &self.case)?;
        write_record("order", &self.order)?;
        write_record("processing", &self.processing)?;
        write_record("io", &self.io)?;

        // Filters
        write_record(
            "min-chars",
            &Self::format_option_str(self.min_chars.as_ref()),
        )?;
        write_record(
            "min-count",
            &Self::format_option_str(self.min_count.as_ref()),
        )?;
        write_record(
            "exclude-words",
            &Self::format_option_str(self.exclude_words),
        )?;
        write_record(
            "exclude-patterns",
            &Self::format_option_str(self.exclude_patterns),
        )?;
        write_record(
            "include-patterns",
            &Self::format_option_str(self.include_patterns),
        )?;

        let buffer = wtr
            .into_inner()
            .with_context(|| "Failed to extract CSV data")?;

        String::from_utf8(buffer).with_context(|| "Failed to convert CSV data to UTF-8")
    }

    /// Log verbose details in text format.
    pub fn log_text(&mut self) -> Result<()> {
        self.write_fields()?;

        // Add a newline separator if the tally has entries
        if self.tally.count() > 0 {
            self.output
                .write_line("\n")
                .with_context(|| "Failed to write newline separator")?;
        }

        Ok(())
    }

    /// Log verbose details in CSV format.
    pub fn log_csv(&mut self) -> Result<()> {
        let csv_data = self.build_csv_data()?;
        self.output
            .write_line(&csv_data)
            .with_context(|| "Failed to write CSV data")?;
        self.output
            .write_line("\n")
            .with_context(|| "Failed to write trailing newline")?;

        Ok(())
    }

    /// Log verbose details in JSON format.
    pub fn log_json(&mut self) -> Result<()> {
        let json = serde_json::to_string(self)
            .with_context(|| "Failed to serialize verbose info to JSON")?;
        self.output
            .write_line(&format!("{json}\n"))
            .with_context(|| "Failed to write JSON output")?;

        Ok(())
    }
}

/// Handle verbose output based on word tally results.
///
/// Outputs information about the word tally in the specified format
/// (JSON, CSV, or plain text) to standard error.
pub fn handle_verbose_output(
    word_tally: &WordTally<'_>,
    format: Format,
    delimiter: &str,
    source: &str,
) -> Result<()> {
    let mut output = Output::stderr();
    let mut verbose = Verbose::new(&mut output, word_tally, delimiter, source);

    match format {
        Format::Json => verbose.log_json()?,
        Format::Csv => verbose.log_csv()?,
        Format::Text => verbose.log_text()?,
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use word_tally::WordTally;

    // Create a mock WordTally for testing verbose output
    fn create_mock_tally() -> WordTally<'static> {
        WordTally::default()
    }

    #[test]
    fn test_handle_verbose_output_ok() {
        let tally = create_mock_tally();
        let result = handle_verbose_output(&tally, Format::Text, " ", "-");
        assert!(result.is_ok());
    }

    #[test]
    fn test_format_enum_as_string() {
        // Test direct string formatting using Display trait
        let case = Case::Lower;
        assert_eq!(case.to_string(), "lower");

        let sort = Sort::Desc;
        assert_eq!(sort.to_string(), "desc");

        let processing = Processing::Sequential;
        assert_eq!(processing.to_string(), "sequential");

        let io = Io::Streamed;
        assert_eq!(io.to_string(), "streamed");
    }

    #[test]
    fn test_format_option() {
        // Test formatting of options with format_option_str
        let some_min_chars: Option<MinChars> = Some(3);
        let formatted = Verbose::format_option_str(some_min_chars.as_ref());
        assert_eq!(formatted, "3");

        let none_min_chars: Option<MinChars> = None;
        let formatted = Verbose::format_option_str(none_min_chars.as_ref());
        assert_eq!(formatted, "none");
    }

    #[test]
    fn test_json_serialization() {
        let tally = create_mock_tally();
        let mut output = Output::stderr();
        let verbose = Verbose::new(&mut output, &tally, " ", "-");

        let json = serde_json::to_string(&verbose)
            .expect("JSON serialization should succeed for Verbose struct");

        // Validate basic JSON structure
        assert!(json.contains("\"source\":\"-\""));
        assert!(json.contains("\"totalWords\":0"));
        assert!(json.contains("\"uniqueWords\":0"));
        assert!(json.contains("\"case\":\"lower\""));
        assert!(json.contains("\"order\":\"desc\""));
        assert!(json.contains("\"processing\":\"sequential\""));
        assert!(json.contains("\"io\":\"streamed\""));
    }

    #[test]
    fn test_build_csv_data() {
        let tally = create_mock_tally();
        let mut output = Output::stderr();
        let verbose = Verbose::new(&mut output, &tally, " ", "-");

        let csv = verbose
            .build_csv_data()
            .expect("CSV data generation should succeed");

        // Validate CSV format
        assert!(csv.contains("metric,value"));
        assert!(csv.contains("source,-"));
        assert!(csv.contains("total-words,0"));
        assert!(csv.contains("unique-words,0"));
        assert!(csv.contains("case,lower"));
        assert!(csv.contains("order,desc"));
        assert!(csv.contains("processing,sequential"));
        assert!(csv.contains("io,streamed"));
    }
}
