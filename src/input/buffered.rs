//! Buffered source for sequential file or stdin input.

use std::{
    fmt::{self, Display, Formatter},
    fs::{self, File},
    io::{self, BufRead, BufReader, Read, Stdin},
    path::{Path, PathBuf},
    sync::Mutex,
};

use anyhow::{Context, Result};

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

impl Buffered {
    /// Reads entire contents using Monoio async I/O for better performance.
    ///
    /// This method uses:
    /// - io_uring on Linux for zero-copy operations
    /// - kqueue on macOS for BSD-style async I/O
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file cannot be opened
    /// - The async runtime fails to initialize
    /// - The read operation fails
    pub fn read_all_async(&self) -> Result<Vec<u8>> {
        match self {
            Self::File(path, _) => {
                // Best practice: Use monoio's optimized fs::read for whole file reading
                monoio::RuntimeBuilder::<monoio::FusionDriver>::new()
                    .build()
                    .map_err(|e| WordTallyError::Config(
                        format!("monoio runtime initialization failed: {e}")
                    ))?
                    .block_on(async {
                        // Using monoio::fs::read is the recommended approach for reading entire files
                        monoio::fs::read(path).await
                            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
                    })
                    .with_context(|| format!("{}: async read failed", path.display()))
            }
            Self::Stdin(_) => {
                // Fall back to sync read for stdin
                let mut buffer = Vec::new();
                self.with_buf_read(|buf_read| {
                    buf_read.read_to_end(&mut buffer)
                        .with_context(|| "failed to read from stdin")
                })??;
                Ok(buffer)
            }
        }
    }
    
    /// Reads entire contents, using async I/O for files.
    pub fn read_all_optimized(&self) -> Result<Vec<u8>> {
        self.read_all_async()
    }
    
    /// Reads file in chunks using Monoio for streaming processing.
    ///
    /// This method is optimized for large files that shouldn't be loaded
    /// entirely into memory. It uses positional reads as recommended by Monoio.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file cannot be opened
    /// - Any read operation fails
    /// - The callback function returns an error
    #[allow(dead_code)]
    pub fn read_chunked_async<F>(&self, chunk_size: usize, mut callback: F) -> Result<()>
    where
        F: FnMut(&[u8]) -> Result<()>,
    {
        match self {
            Self::File(path, _) => {
                monoio::RuntimeBuilder::<monoio::FusionDriver>::new()
                    .build()
                    .map_err(|e| WordTallyError::Config(
                        format!("monoio runtime initialization failed: {e}")
                    ))?
                    .block_on(async {
                        let file = monoio::fs::File::open(path).await
                            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                        
                        let mut offset = 0u64;
                        loop {
                            let buffer = vec![0u8; chunk_size];
                            
                            // Use positional read_at as recommended by Monoio docs
                            let (res, buf) = file.read_at(buffer, offset).await;
                            match res {
                                Ok(0) => break, // EOF
                                Ok(n) => {
                                    callback(&buf[..n])
                                        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                                    offset += n as u64;
                                }
                                Err(e) => {
                                    // Best practice: Close file even on error
                                    drop(file.close().await);
                                    return Err(e);
                                }
                            }
                        }
                        
                        // Best practice: Explicitly close the file
                        file.close().await?;
                        Ok(())
                    })
                    .with_context(|| format!("{}: async chunked read failed", path.display()))
            }
            Self::Stdin(_) => {
                // Fall back to sync chunked read for stdin
                self.with_buf_read(|buf_read| {
                    let mut buffer = vec![0u8; chunk_size];
                    loop {
                        match Read::read(buf_read, &mut buffer)? {
                            0 => break,
                            n => callback(&buffer[..n])?,
                        }
                    }
                    Ok(())
                })
                .with_context(|| "failed to read chunks from stdin")?
            }
        }
    }
}
