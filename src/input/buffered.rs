//! Buffered source for sequential file or stdin input.

use std::{
    fmt::{self, Display, Formatter},
    fs::{self, File},
    io::{self, BufRead, BufReader, Stdin},
    path::{Path, PathBuf},
    sync::Mutex,
};

use anyhow::Result;

use super::{Metadata, open_file_with_error_context};
use crate::{WordTallyError, error::Error};

/// Buffered source for sequential file or stdin input.
#[derive(Debug)]
pub enum Buffered {
    /// Standard input reader.
    Stdin(Mutex<BufReader<Stdin>>),
    /// File reader.
    File(PathBuf, Mutex<BufReader<File>>),
}

impl Buffered {
    /// Creates a reader for standard input.
    #[must_use]
    pub fn stdin() -> Self {
        Self::Stdin(Mutex::new(BufReader::new(io::stdin())))
    }

    /// Provides access to the underlying buffered reader.
    ///
    /// # Errors
    ///
    /// Returns `Error::MutexPoisoned` if the mutex was poisoned by a panic in another thread.
    pub fn with_buf_read<R>(&self, f: impl FnOnce(&mut dyn BufRead) -> R) -> Result<R> {
        match self {
            Self::Stdin(buf_reader) => Self::with_mutex(buf_reader, f),
            Self::File(_, buf_reader) => Self::with_mutex(buf_reader, f),
        }
    }

    /// Helper to safely access the mutex-protected reader.
    fn with_mutex<T: BufRead, R>(
        mutex: &Mutex<T>,
        f: impl FnOnce(&mut dyn BufRead) -> R,
    ) -> Result<R> {
        mutex
            .lock()
            .map(|mut guard| f(&mut *guard))
            .map_err(|_| Error::MutexPoisoned.into())
    }
}

impl Display for Buffered {
    /// Formats the reader for display.
    /// Shows file path or "-" for stdin.
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::File(path, _) => write!(f, "{}", path.display()),
            Self::Stdin(_) => write!(f, "-"),
        }
    }
}

impl TryFrom<&Path> for Buffered {
    type Error = WordTallyError;

    /// Creates a reader from a file path or stdin.
    ///
    /// Use `"-"` for stdin input.
    ///
    /// # Errors
    ///
    /// Returns `WordTallyError::Io` with specific messages for:
    /// - File not found
    /// - Permission denied
    /// - Other I/O errors
    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        if path.as_os_str() == "-" {
            Ok(Self::stdin())
        } else {
            let file = open_file_with_error_context(path)?;
            let path_buf = path.to_path_buf();
            Ok(Self::File(path_buf, Mutex::new(BufReader::new(file))))
        }
    }
}

impl TryFrom<PathBuf> for Buffered {
    type Error = WordTallyError;

    /// Creates a reader from a path buffer.
    ///
    /// The path "-" is interpreted as stdin.
    ///
    /// # Errors
    ///
    /// Returns `WordTallyError::Io` if the file cannot be opened.
    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
        Self::try_from(path.as_path())
    }
}

impl TryFrom<&str> for Buffered {
    type Error = WordTallyError;

    /// Creates a reader from a string path.
    ///
    /// The path "-" is interpreted as stdin.
    ///
    /// # Errors
    ///
    /// Returns `WordTallyError::Io` if the file cannot be opened.
    fn try_from(path: &str) -> Result<Self, Self::Error> {
        Self::try_from(Path::new(path))
    }
}

impl Metadata for Buffered {
    /// Returns the file path for file readers, `None` for stdin.
    fn path(&self) -> Option<&Path> {
        match self {
            Self::File(path, _) => Some(path),
            Self::Stdin(_) => None,
        }
    }

    /// Returns the file size in bytes for file readers, `None` for stdin.
    fn size(&self) -> Option<u64> {
        self.path()
            .and_then(|path| fs::metadata(path).ok().map(|metadata| metadata.len()))
    }
}
