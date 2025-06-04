//! Input sources for files, stdin and memory-mapped I/O.

use crate::WordTallyError;
use crate::input_reader::InputReader;
use crate::options::io::Io;
use anyhow::Result;
use memmap2::Mmap;
use std::fmt::{self, Formatter};
use std::fs::{self, File};
use std::path::{Path, PathBuf};

/// `Input` to read from a file, stdin, memory-mapped source, or bytes.
#[derive(Debug)]
pub enum Input {
    /// Read from standard input.
    Stdin,
    /// Read from a file.
    File(PathBuf),
    /// Read from a memory-mapped file.
    MemoryMap(Mmap, PathBuf),
    /// Read from in-memory bytes.
    Bytes(Box<[u8]>),
}

impl Input {
    /// Construct an `Input` from a file path or stdin (designated by "-").
    ///
    /// For bytes data, use `Input::from_bytes` instead.
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
    /// - `Io::ParallelBytes` is specified (use `Input::from_bytes` instead)
    pub fn new<P: AsRef<Path>>(path: P, io: Io) -> Result<Self> {
        // Handle the stdin case
        let path_ref = path.as_ref();
        if path_ref.as_os_str() == "-" {
            match io {
                Io::ParallelMmap => return Err(WordTallyError::MmapStdin.into()),
                _ => return Ok(Self::Stdin),
            }
        }

        match io {
            Io::Stream | Io::ParallelStream | Io::ParallelInMemory => {
                let path_buf = path_ref.to_path_buf();

                Ok(Self::File(path_buf))
            }
            Io::ParallelMmap => {
                let path_buf = path_ref.to_path_buf();
                let file = File::open(&path_buf).map_err(|e| WordTallyError::Io {
                    path: path_buf.display().to_string(),
                    message: "failed to open file for memory mapping".to_string(),
                    source: e,
                })?;

                // Safety: Memory mapping requires `unsafe` per memmap2 crate
                #[allow(unsafe_code)]
                let mmap = unsafe { Mmap::map(&file) }.map_err(|e| WordTallyError::Io {
                    path: path_buf.display().to_string(),
                    message: "failed to create memory map".to_string(),
                    source: e,
                })?;

                Ok(Self::MemoryMap(mmap, path_buf))
            }
            Io::ParallelBytes => Err(WordTallyError::BytesWithPath.into()),
        }
    }

    /// Create an `Input` from byte data.
    ///
    /// # Examples
    ///
    /// ```
    /// use word_tally::Input;
    ///
    /// let text = "hello world hello";
    /// let input = Input::from_bytes(text);
    /// ```
    pub fn from_bytes<B: AsRef<[u8]>>(bytes: B) -> Self {
        Self::Bytes(bytes.as_ref().into())
    }

    /// A helper to get an `InputReader` from this input.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - File cannot be opened for reading
    /// - Standard input cannot be accessed
    /// - Any other I/O error occurs during reader creation
    pub fn reader(&self) -> Result<InputReader<'_>, WordTallyError> {
        InputReader::new(self)
    }

    /// Returns the source path or identifier for display in error messages.
    #[must_use]
    pub fn source(&self) -> String {
        match self {
            Self::Stdin => "-".to_string(),
            Self::File(path) | Self::MemoryMap(_, path) => path.display().to_string(),
            Self::Bytes(_) => "<bytes>".to_string(),
        }
    }

    /// Get the size of the input in bytes, if available.
    /// Returns `None` for stdin and when a filesize can't be determined.
    #[must_use]
    pub fn size(&self) -> Option<u64> {
        match self {
            Self::Stdin => None,
            Self::File(path) => fs::metadata(path).ok().map(|metadata| metadata.len()),
            Self::MemoryMap(mmap, _) => Some(mmap.len() as u64),
            Self::Bytes(bytes) => Some(bytes.len() as u64),
        }
    }
}

impl Default for Input {
    fn default() -> Self {
        Self::Stdin
    }
}

impl From<Box<[u8]>> for Input {
    fn from(bytes: Box<[u8]>) -> Self {
        Self::Bytes(bytes)
    }
}

impl From<Vec<u8>> for Input {
    fn from(bytes: Vec<u8>) -> Self {
        Self::Bytes(bytes.into_boxed_slice())
    }
}

impl fmt::Display for Input {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Stdin => write!(f, "Stdin"),
            Self::File(path) => write!(f, "File({})", path.display()),
            Self::MemoryMap(_, path) => write!(f, "Mmap({})", path.display()),
            Self::Bytes(_) => write!(f, "Bytes"),
        }
    }
}
