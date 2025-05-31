//! Write trait abstractions for stdout and file serialization.

use crate::options::serialization::Serialization;
use crate::{Count, Word, WordTally, WordTallyError};
use anyhow::{Context, Result};
use std::fmt::{self, Debug, Formatter};
use std::fs::File;
use std::io::{self, ErrorKind::BrokenPipe, LineWriter, Write};
use std::path::Path;

/// `Writer` dynamic dispatches the `Write` trait.
pub type Writer = Box<dyn Write>;

/// `Output` writes to either a file or stream like stdout or stderr.
pub struct Output {
    writer: Writer,
}

impl Default for Output {
    /// Default output is stdout.
    fn default() -> Self {
        Self::stdout()
    }
}

impl Debug for Output {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Output")
            .field("writer", &"<dyn Write>")
            .finish()
    }
}

/// `Write` trait implementation for our writer.
///
/// Note: Broken pipe handling is needed for commands like `word-tally | head`.
/// Once it stabilizes, the `unix_sigpipe` feature will be a better solution here.
impl Write for Output {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self.writer.write(buf) {
            Ok(n) => Ok(n),
            Err(err) => match err.kind() {
                // Pretend we wrote all bytes on broken pipe
                BrokenPipe => Ok(buf.len()),
                _ => Err(err),
            },
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self.writer.flush() {
            Ok(()) => Ok(()),
            Err(err) => match err.kind() {
                // Ignore broken pipe errors for CLI tools
                BrokenPipe => Ok(()),
                _ => Err(err),
            },
        }
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        match self.writer.write_all(buf) {
            Ok(()) => Ok(()),
            Err(err) => match err.kind() {
                // Ignore broken pipe errors for CLI tools
                BrokenPipe => Ok(()),
                _ => Err(err),
            },
        }
    }
}

impl Output {
    /// Creates an `Output` from optional arguments, choosing between file or stdout.
    ///
    /// # Errors
    ///
    /// Returns an error if the output file path is provided but the file cannot be created.
    pub fn new(output: Option<&Path>) -> Result<Self> {
        match output {
            Some(path) if path == Path::new("-") => Ok(Self::stdout()),
            Some(path) => Self::file(path),
            None => Ok(Self::stdout()),
        }
    }

    /// Creates an `Output` that writes to a file with error context.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be created, which could happen due to:
    /// - Path doesn't exist
    /// - Permission issues
    /// - Disk is full
    /// - File is already in use by another process
    pub fn file(path: &Path) -> Result<Self> {
        let file = File::create(path)
            .map(|file| Box::new(LineWriter::new(file)))
            .with_context(|| format!("failed to create output file: {}", path.display()))?;

        Ok(Self { writer: file })
    }

    /// Creates an `Output` that writes to stdout.
    #[must_use]
    pub fn stdout() -> Self {
        Self {
            writer: Box::new(io::stdout().lock()),
        }
    }

    /// Creates an `Output` that writes to stderr.
    #[must_use]
    pub fn stderr() -> Self {
        Self {
            writer: Box::new(io::stderr().lock()),
        }
    }

    /// Writes a chunk of text to the writer.
    ///
    /// # Errors
    ///
    /// Returns an I/O error if the write operation fails, such as when:
    /// - Disk is full
    /// - File is not writable
    /// - Pipe is broken
    pub fn write_chunk(&mut self, chunk: &str) -> io::Result<()> {
        self.write_all(chunk.as_bytes())
    }

    /// Writes word tally data in the specified format.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Writing to the output fails due to I/O errors
    /// - CSV or JSON serialization encounters an error
    pub fn write_formatted_tally(&mut self, word_tally: &WordTally<'_>) -> Result<()> {
        let serialization = word_tally.options().serialization();
        let tally = word_tally.tally();

        match serialization {
            Serialization::Text {
                field_delimiter,
                entry_delimiter,
            } => self.write_text(tally, field_delimiter.as_str(), entry_delimiter.as_str()),
            Serialization::Json => self.write_json(tally),
            Serialization::Csv => self.write_csv(tally),
        }
    }

    fn write_text(
        &mut self,
        tally: &[(Word, Count)],
        field_delimiter: &str,
        entry_delimiter: &str,
    ) -> Result<()> {
        tally.iter().try_for_each(|(word, count)| {
            self.write_chunk(&format!("{word}{field_delimiter}{count}{entry_delimiter}"))
                .context("failed to write word-count pair")
        })?;

        self.flush().context("failed to flush output")
    }

    fn write_json(&mut self, tally: &[(Word, Count)]) -> Result<()> {
        let mut json_data = Vec::with_capacity(tally.len());
        json_data.extend(tally.iter().map(|(word, count)| (word.as_ref(), count)));
        let json = serde_json::to_string(&json_data).map_err(WordTallyError::JsonSerialization)?;
        self.write_chunk(&format!("{json}\n"))
            .context("failed to write JSON output")?;

        self.flush().context("failed to flush JSON output")
    }

    fn write_csv(&mut self, tally: &[(Word, Count)]) -> Result<()> {
        let mut csv_writer = csv::Writer::from_writer(&mut self.writer);
        csv_writer
            .write_record(["word", "count"])
            .map_err(WordTallyError::CsvSerialization)?;

        tally.iter().try_for_each(|(word, count)| {
            csv_writer
                .write_record([word.as_ref(), &count.to_string()])
                .map_err(WordTallyError::CsvSerialization)
        })?;

        csv_writer.flush().context("failed to flush CSV output")
    }
}
