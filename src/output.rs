use anyhow::{Context, Result};
use std::fs::File;
use std::io::{self, ErrorKind::BrokenPipe, LineWriter, Write};
use std::path::PathBuf;

/// `Writer` is a boxed type for dynamic dispatch of the `Write` trait.
pub type Writer = Box<dyn Write>;

/// `Output` to a file or pipe.
pub struct Output {
    writer: Writer,
}

impl Output {
    /// Construct an `Output` to a file with error context.
    pub fn file(path: PathBuf) -> Result<Self> {
        let file = File::create(&path)
            .map(|file| Box::new(LineWriter::new(file)) as Writer)
            .with_context(|| format!("Failed to create file: {:?}", path))?;
        Ok(Self { writer: file })
    }

    /// Constructs an `Output` to stdout.
    pub fn stdout() -> Self {
        Self {
            writer: Box::new(io::stdout().lock()),
        }
    }

    /// Constructs an `Output` to stderr.
    pub fn stderr() -> Self {
        Self {
            writer: Box::new(io::stderr().lock()),
        }
    }

    /// Constructs an `Output` from optional arguments, choosing between file or stdout.
    pub fn from_args(output: Option<PathBuf>) -> Result<Self> {
        match output.as_deref().map(|p| p.to_string_lossy()) {
            Some(ref path) if path == "-" => Ok(Self::stdout()),
            Some(path) => Self::file(PathBuf::from(path.as_ref())),
            None => Ok(Self::stdout()),
        }
    }

    /// Writes a line to the writer, handling pipe-related errors.
    pub fn write_line(&mut self, line: &str) -> Result<()> {
        Self::handle_broken_pipe(self.writer.write_all(line.as_bytes()))
    }

    /// Flushes the writer, ensuring all output is written.
    pub fn flush(&mut self) -> Result<()> {
        Self::handle_broken_pipe(self.writer.flush())
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
