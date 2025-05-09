use crate::formatting::Format;
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
    /// Default output is stdout
    fn default() -> Self {
        Self::stdout()
    }
}

impl Write for Output {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.writer.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
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
            .map(|file| Box::new(LineWriter::new(file)) as Writer)
            .with_context(|| format!("Failed to create output file: {}", path.display()))?;
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

    /// Writes a line to the writer, handling `BrokenPipe` errors gracefully.
    pub fn write_line(&mut self, line: &str) -> Result<()> {
        Self::handle_broken_pipe(self.writer.write_all(line.as_bytes()))
    }

    /// Flushes the writer, ensuring all output is written.
    pub fn flush(&mut self) -> Result<()> {
        Self::handle_broken_pipe(self.writer.flush())
    }

    /// Writes word tally data in the specified format.
    pub fn write_formatted_tally(
        &mut self,
        word_data: &[(Box<str>, usize)],
        format: Format,
        delimiter: &str,
    ) -> Result<()> {
        match format {
            Format::Text => {
                for (word, count) in word_data {
                    self.write_line(&format!("{word}{delimiter}{count}\n"))?;
                }
            }
            Format::Json => {
                let json = serde_json::to_string(
                    &word_data
                        .iter()
                        .map(|(word, count)| (word.as_ref(), count))
                        .collect::<Vec<_>>(),
                )
                .with_context(|| "Failed to serialize word tally to JSON")?;
                self.write_line(&format!("{json}\n"))?;
            }
            Format::Csv => {
                let mut wtr = csv::Writer::from_writer(Vec::new());
                wtr.write_record(["word", "count"])?;
                for (word, count) in word_data {
                    wtr.write_record([word.as_ref(), &count.to_string()])?;
                }
                let csv_data = String::from_utf8(wtr.into_inner()?)
                    .with_context(|| "Failed to convert CSV output to UTF-8 string")?;
                self.write_line(&csv_data)?;
            }
        }

        self.flush()
    }

    /// Processes the result of a write, handling `BrokenPipe` errors gracefully.
    fn handle_broken_pipe(result: io::Result<()>) -> Result<()> {
        match result {
            Ok(()) => Ok(()),
            Err(err) => match err.kind() {
                BrokenPipe => Ok(()),
                _ => Err(err.into()),
            },
        }
    }
}
