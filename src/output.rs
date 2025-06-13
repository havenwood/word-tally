//! Output writing for stdout and file serialization.

use std::{
    fmt::{self, Debug, Formatter},
    fs::File,
    io::{self, ErrorKind::BrokenPipe, LineWriter, Write},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};

use crate::{Count, Word, WordTally, WordTallyError, options::serialization::Serialization};

/// `Output` writes to either a file or stream like stdout or stderr.
pub struct Output {
    writer: Box<dyn io::Write>,
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
impl io::Write for Output {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self.writer.write(buf) {
            Ok(n) => Ok(n),
            Err(e) if e.kind() == BrokenPipe => Ok(buf.len()),
            Err(e) => Err(e),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self.writer.flush() {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == BrokenPipe => Ok(()),
            Err(e) => Err(e),
        }
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        match self.writer.write_all(buf) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == BrokenPipe => Ok(()),
            Err(e) => Err(e),
        }
    }
}

impl Output {
    /// Creates an `Output` from an optional path.
    /// - `None` -> stdout
    /// - `Some("-")` -> stdout
    /// - `Some(path)` -> file
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use word_tally::Output;
    /// use std::path::Path;
    ///
    /// // Write to stdout (default)
    /// let stdout = Output::new(None)?;
    ///
    /// // Explicitly write to stdout with "-"
    /// let explicit_stdout = Output::new(Some(Path::new("-")))?;
    ///
    /// // Write to a file
    /// let file_output = Output::new(Some(Path::new("results.txt")))?;
    /// # Ok::<(), anyhow::Error>(())
    /// ```
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

    /// Writes word tally data in the specified format.
    ///
    /// # Examples
    ///
    /// ```
    /// use word_tally::{WordTally, TallyMap, View, Output, Options, Serialization, Io};
    ///
    /// // Create a tally and write it as JSON to stdout
    /// let view = View::from("hello world hello".as_bytes());
    /// let options = Options::default()
    ///     .with_io(Io::ParallelBytes)
    ///     .with_serialization(Serialization::Json);
    /// let tally_map = TallyMap::from_view(&view, &options)?;
    /// let tally = WordTally::from_tally_map(tally_map, &options);
    ///
    /// let mut output = Output::new(None)?; // Write to stdout
    /// output.write_formatted_tally(&tally)?;
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Writing to the output fails due to I/O errors
    /// - CSV or JSON serialization encounters an error
    pub fn write_formatted_tally(&mut self, word_tally: &WordTally) -> Result<()> {
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
            self.write_all(format!("{word}{field_delimiter}{count}{entry_delimiter}").as_bytes())
                .context("failed to write word-count pair")
        })?;

        self.flush().context("failed to flush output")
    }

    fn write_json(&mut self, tally: &[(Word, Count)]) -> Result<()> {
        let mut json_data = Vec::with_capacity(tally.len());
        json_data.extend(tally.iter().map(|(word, count)| (word.as_ref(), count)));
        let json = serde_json::to_string(&json_data).map_err(WordTallyError::JsonSerialization)?;
        self.write_all(format!("{json}\n").as_bytes())
            .context("failed to write JSON output")?;

        self.flush().context("failed to flush JSON output")
    }

    fn write_csv(&mut self, tally: &[(Word, Count)]) -> Result<()> {
        let mut csv_writer = csv::Writer::from_writer(self);
        csv_writer
            .write_record(["word", "count"])
            .map_err(WordTallyError::CsvSerialization)?;

        for (word, count) in tally {
            csv_writer
                .write_record([word.as_ref(), &count.to_string()])
                .map_err(WordTallyError::CsvSerialization)?;
        }

        csv_writer.flush().context("failed to flush CSV output")
    }
}

/// Allows creating `Output` from an optional path (`None` = stdout).
impl TryFrom<Option<&PathBuf>> for Output {
    type Error = WordTallyError;

    fn try_from(path: Option<&PathBuf>) -> Result<Self, Self::Error> {
        path.map_or_else(|| Ok(Self::stdout()), |path| Self::try_from(path.as_path()))
    }
}

/// Allows creating `Output` from a path (stdin for "-").
impl TryFrom<&Path> for Output {
    type Error = WordTallyError;

    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        if path == Path::new("-") {
            Ok(Self::stdout())
        } else {
            Self::file(path).map_err(|e| WordTallyError::Io {
                path: path.display().to_string(),
                message: "failed to create output file".to_string(),
                source: e.downcast_ref::<io::Error>().map_or_else(
                    || io::Error::other(e.to_string()),
                    |io_err| io::Error::new(io_err.kind(), io_err.to_string()),
                ),
            })
        }
    }
}
