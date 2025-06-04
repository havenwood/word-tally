//! Input sources for files, stdin and memory-mapped I/O.

use std::fmt::{self, Formatter};
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Stdin};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use anyhow::Result;
use memmap2::Mmap;

use crate::WordTallyError;
use crate::options::io::Io;

/// Direct view into memory-mapped or in-memory data.
#[derive(Debug)]
pub enum View {
    /// View into a memory-mapped file.
    Mmap { path: PathBuf, mmap: Mmap },
    /// View into in-memory bytes.
    Bytes(Box<[u8]>),
}

impl View {
    /// Get the path if this view is file-based.
    #[must_use]
    pub fn path(&self) -> Option<&Path> {
        match self {
            Self::Mmap { path, .. } => Some(path),
            Self::Bytes(_) => None,
        }
    }

    /// Get the size of the view in bytes.
    #[must_use]
    pub fn size(&self) -> u64 {
        self.as_ref().len() as u64
    }
}

impl AsRef<[u8]> for View {
    fn as_ref(&self) -> &[u8] {
        match self {
            Self::Mmap { mmap, .. } => mmap,
            Self::Bytes(bytes) => bytes,
        }
    }
}

/// Sequential reader for stdin or file input.
#[derive(Debug)]
pub enum Reader {
    /// Read from standard input.
    Stdin(Mutex<BufReader<Stdin>>),
    /// Read from a file.
    File(PathBuf, Mutex<BufReader<File>>),
}

impl Reader {
    /// Execute a closure with mutable access to the underlying `BufReader`.
    ///
    /// # Panics
    ///
    /// Panics if the mutex protecting the `BufReader` is poisoned.
    /// This should only occur if a thread panicked while holding the lock.
    pub fn with_buf_read<R>(&self, f: impl FnOnce(&mut dyn BufRead) -> R) -> R {
        match self {
            Self::Stdin(buf_reader) => Self::with_mutex(buf_reader, f),
            Self::File(_, buf_reader) => Self::with_mutex(buf_reader, f),
        }
    }

    /// Helper to apply a closure with a locked mutex guard.
    fn with_mutex<T: BufRead, R>(mutex: &Mutex<T>, f: impl FnOnce(&mut dyn BufRead) -> R) -> R {
        f(&mut *mutex.lock().expect("mutex should be unpoisoned"))
    }

    /// Get the path if this reader is file-based.
    #[must_use]
    pub fn path(&self) -> Option<&Path> {
        match self {
            Self::File(path, _) => Some(path),
            Self::Stdin(_) => None,
        }
    }
}

/// Input source that can be either a sequential reader or a memory view.
#[derive(Debug)]
pub enum Input {
    /// Direct memory view (mmap or bytes).
    View(View),
    /// Sequential reader (stdin or file).
    Reader(Reader),
}

impl Input {
    /// Construct an `Input` from a file path or stdin (designated by "-").
    ///
    /// For bytes data, use `Input::from()` instead.
    ///
    /// # Examples
    ///
    /// ```
    /// use word_tally::{Input, Io};
    ///
    /// # fn example() -> anyhow::Result<()> {
    /// // Read from a file with default parallel stream
    /// let file_input = Input::new("document.txt", Io::default())?;
    ///
    /// // Read from stdin
    /// let stdin_input = Input::new("-", Io::default())?;
    ///
    /// // Fast processing with memory mapping
    /// let mmap_input = Input::new("large_file.txt", Io::ParallelMmap)?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - File cannot be opened for reading (when using file-based I/O modes)
    /// - Memory mapping fails (when using memory-mapped I/O)
    /// - `Io::ParallelBytes` is specified (use `Input::from()` instead)
    pub fn new<P: AsRef<Path>>(path: P, io: Io) -> Result<Self> {
        let path_ref = path.as_ref();

        // Handle stdin case
        if path_ref.as_os_str() == "-" {
            return match io {
                Io::ParallelMmap => Err(WordTallyError::MmapStdin.into()),
                _ => Ok(Self::Reader(Reader::Stdin(Mutex::new(BufReader::new(
                    io::stdin(),
                ))))),
            };
        }

        match io {
            Io::Stream | Io::ParallelStream | Io::ParallelInMemory => {
                let path_buf = path_ref.to_path_buf();
                let file = Self::open_file(&path_buf)?;

                Ok(Self::Reader(Reader::File(
                    path_buf,
                    Mutex::new(BufReader::new(file)),
                )))
            }
            Io::ParallelMmap => {
                let path_buf = path_ref.to_path_buf();
                let file = Self::open_file(&path_buf)?;

                // Safety: Memory mapping requires `unsafe` per memmap2 crate
                #[allow(unsafe_code)]
                let mmap = unsafe { Mmap::map(&file) }.map_err(|e| WordTallyError::Io {
                    path: path_buf.display().to_string(),
                    message: "failed to create memory map".to_string(),
                    source: e,
                })?;

                Ok(Self::View(View::Mmap {
                    path: path_buf,
                    mmap,
                }))
            }
            Io::ParallelBytes => Err(WordTallyError::BytesWithPath.into()),
        }
    }

    /// Opens a file with detailed error messages based on the error kind.
    fn open_file(path: &Path) -> Result<File, WordTallyError> {
        File::open(path).map_err(|source| {
            let message = match source.kind() {
                io::ErrorKind::NotFound => format!("no such file: {}", path.display()),
                io::ErrorKind::PermissionDenied => {
                    format!("permission denied: {}", path.display())
                }
                _ => format!("failed to open file: {}", path.display()),
            };

            WordTallyError::Io {
                path: path.display().to_string(),
                message,
                source,
            }
        })
    }

    /// Get the file path if this input is file-based.
    /// Returns `None` for stdin and bytes inputs.
    #[must_use]
    pub fn path(&self) -> Option<&Path> {
        match self {
            Self::Reader(reader) => reader.path(),
            Self::View(view) => view.path(),
        }
    }

    /// Returns a display string for error messages and logging.
    /// This is "-" for stdin, the file path for files, and "<bytes>" for byte data.
    #[must_use]
    pub fn source(&self) -> String {
        match self {
            Self::Reader(reader) => match reader {
                Reader::File(path, _) => path.display().to_string(),
                Reader::Stdin(_) => String::from("-"),
            },
            Self::View(view) => view.path().map_or_else(
                || String::from("<bytes>"),
                |path| path.display().to_string(),
            ),
        }
    }

    /// Get the size of the input in bytes, if available.
    /// Returns `None` for stdin and when a filesize can't be determined.
    #[must_use]
    pub fn size(&self) -> Option<u64> {
        match self {
            Self::Reader(reader) => match reader {
                Reader::Stdin(_) => None,
                Reader::File(path, _) => fs::metadata(path).ok().map(|metadata| metadata.len()),
            },
            Self::View(view) => Some(view.size()),
        }
    }
}

impl Default for Input {
    fn default() -> Self {
        Self::Reader(Reader::Stdin(Mutex::new(BufReader::new(io::stdin()))))
    }
}

impl From<Box<[u8]>> for Input {
    fn from(bytes: Box<[u8]>) -> Self {
        Self::View(View::Bytes(bytes))
    }
}

impl From<&[u8]> for Input {
    fn from(bytes: &[u8]) -> Self {
        Self::View(View::Bytes(Box::from(bytes)))
    }
}

impl From<Vec<u8>> for Input {
    fn from(bytes: Vec<u8>) -> Self {
        Self::View(View::Bytes(bytes.into_boxed_slice()))
    }
}

impl fmt::Display for Input {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Reader(reader) => match reader {
                Reader::Stdin(_) => write!(f, "Stdin"),
                Reader::File(path, _) => write!(f, "File({})", path.display()),
            },
            Self::View(view) => match view {
                View::Mmap { path, .. } => write!(f, "Mmap({})", path.display()),
                View::Bytes(_) => write!(f, "Bytes"),
            },
        }
    }
}
