use anyhow::{Context, Result};
use std::fs::File;
use std::io::{self, ErrorKind::BrokenPipe, LineWriter, Write};
use std::path::{Path, PathBuf};

/// `Writer` dynamic dispatches the `Write` trait.
pub type Writer = Box<dyn Write>;

/// `Output` writes to either a file or stream like stdout or stderr.
pub struct Output {
    writer: Writer,
}

impl Output {
    /// Creates an `Output` that writes to a file with error context.
    pub fn file(path: PathBuf) -> Result<Self> {
        let file = File::create(&path)
            .map(|file| Box::new(LineWriter::new(file)) as Writer)
            .with_context(|| format!("Failed to create file: {:?}", path))?;
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

    /// Creates an `Output` from optional arguments, choosing between file or stdout.
    pub fn from_args(output: &Option<PathBuf>) -> Result<Self> {
        match output.as_deref() {
            Some(path) if path == Path::new("-") => Ok(Self::stdout()),
            Some(path) => Self::file(path.to_path_buf()),
            None => Ok(Self::stdout()),
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
