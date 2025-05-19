//! Write trait abstractions for stdout and file serialization.

use crate::WordTally;
use crate::options::serialization::Format;
use anyhow::{Context, Result};
use std::fmt::{self, Debug, Formatter};
use std::fs::File;
use std::io::{self, ErrorKind::BrokenPipe, LineWriter, Write};
use std::path::{Path, PathBuf};

/// `Writer` dynamic dispatches the `Write` trait.
pub type Writer = Box<dyn Write>;

/// `Output` writes to either a file or stream like stdout or stderr.
pub struct Output {
    writer: Writer,
}

impl Debug for Output {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Output")
            .field("writer", &"<dyn Write>")
            .finish()
    }
}

impl Default for Output {
    /// Default output is stdout.
    fn default() -> Self {
        Self::stdout()
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
    pub fn new(output: &Option<PathBuf>) -> Result<Self> {
        match output.as_deref() {
            Some(path) if path == Path::new("-") => Ok(Self::stdout()),
            Some(path) => Self::file(path),
            None => Ok(Self::stdout()),
        }
    }

    /// Creates an `Output` that writes to a file with error context.
    pub fn file(path: &Path) -> Result<Self> {
        let file = File::create(path)
            .map(|file| Box::new(LineWriter::new(file)))
            .with_context(|| format!("failed to create output file: {}", path.display()))?;

        Ok(Self { writer: file })
    }

    /// Creates an `Output` that writes to stdout.
    pub fn stdout() -> Self {
        Self {
            writer: Box::new(io::stdout().lock()),
        }
    }

    /// Creates an `Output` that writes to stderr.
    pub fn stderr() -> Self {
        Self {
            writer: Box::new(io::stderr().lock()),
        }
    }

    /// Writes a chunk of text to the writer.
    pub fn write_chunk(&mut self, chunk: &str) -> io::Result<()> {
        self.write_all(chunk.as_bytes())
    }

    /// Writes word tally data in the specified format.
    pub fn write_formatted_tally<'a>(&mut self, word_tally: &WordTally<'a>) -> Result<()> {
        let format = word_tally.options().serialization().format();
        let delimiter = word_tally.options().serialization().delimiter();
        let tally = word_tally.tally();

        match format {
            Format::Text => {
                for (word, count) in tally {
                    self.write_chunk(&format!("{word}{delimiter}{count}\n"))
                        .context("failed to write word-count pair")?;
                }
            }
            Format::Json => {
                let mut json_data = Vec::with_capacity(tally.len());
                json_data.extend(tally.iter().map(|(word, count)| (word.as_ref(), count)));
                let json = serde_json::to_string(&json_data)
                    .context("failed to serialize word tally to JSON")?;
                self.write_chunk(&format!("{json}\n"))
                    .context("failed to write JSON output")?;
            }
            Format::Csv => {
                // The minimum CSV row size is 4 ("a,1\n")
                let min_row_len = tally.len() * 3;
                // The CSV header size is always 11 ("word,count\n")
                let min_capacity = min_row_len + 11;
                let mut wtr = csv::Writer::from_writer(Vec::with_capacity(min_capacity));
                wtr.write_record(["word", "count"])?;
                for (word, count) in tally {
                    wtr.write_record([word.as_ref(), &count.to_string()])?;
                }
                let csv_data = String::from_utf8(wtr.into_inner()?)
                    .context("failed to convert CSV output to UTF-8 string")?;
                self.write_chunk(&csv_data)
                    .context("failed to write CSV output")?;
            }
        }

        self.flush().context("failed to flush output")
    }
}
