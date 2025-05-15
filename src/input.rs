//! Read trait abstractions for files, stdin or memory-mapped I/O.

use crate::io::Io;
use anyhow::{Context, Result};
use memmap2::Mmap;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// A reader that allows memory-mapped files to be used with the standard Read trait.
/// This implementation wraps an Arc<Mmap> for thread-safe access.
#[derive(Clone, Debug)]
pub struct MmapReader {
    mmap: Arc<Mmap>,
    position: usize,
}

/// A cursor-based memory-mapped reader for sequential counting
impl MmapReader {
    pub fn new(mmap: &Arc<Mmap>) -> Self {
        Self {
            mmap: Arc::clone(mmap),
            position: 0,
        }
    }
}

impl Read for MmapReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let remaining = self.mmap.len().saturating_sub(self.position);

        if remaining == 0 {
            return Ok(0);
        }

        let amt = remaining.min(buf.len());

        buf[..amt].copy_from_slice(&self.mmap[self.position..][..amt]);
        self.position += amt;

        Ok(amt)
    }
}

/// A reader that provides a zero-copy Read implementation for byte slices.
/// This implementation holds an Arc<[u8]> for thread-safe access.
#[derive(Clone, Debug)]
pub struct BytesReader {
    bytes: Arc<[u8]>,
    position: usize,
}

impl BytesReader {
    pub fn new(bytes: &Arc<[u8]>) -> Self {
        Self {
            bytes: Arc::clone(bytes),
            position: 0,
        }
    }
}

impl Read for BytesReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let remaining = self.bytes.len().saturating_sub(self.position);

        if remaining == 0 {
            return Ok(0);
        }

        let amt = remaining.min(buf.len());

        buf[..amt].copy_from_slice(&self.bytes[self.position..][..amt]);
        self.position += amt;

        Ok(amt)
    }
}

/// `Input` to read from a file, stdin, memory-mapped source, or bytes.
#[derive(Clone, Debug)]
pub enum Input {
    Stdin,
    File(PathBuf),
    Mmap(Arc<Mmap>, PathBuf),
    Bytes(Arc<[u8]>),
}

impl Default for Input {
    fn default() -> Self {
        Self::Stdin
    }
}

impl std::fmt::Display for Input {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Stdin => write!(f, "Stdin"),
            Self::File(path) => write!(f, "File({})", path.display()),
            Self::Mmap(_, path) => write!(f, "Mmap({})", path.display()),
            Self::Bytes(_) => write!(f, "Bytes"),
        }
    }
}

/// A wrapper that implements Read and BufRead for each input type
/// when read() is called multiple times with state tracked between calls
#[derive(Debug)]
pub enum InputReader {
    Stdin(BufReader<io::Stdin>),
    File(BufReader<File>),
    Mmap(BufReader<MmapReader>),
    Bytes(BufReader<BytesReader>),
}

impl InputReader {
    /// Create a new InputReader from an Input
    pub fn new(input: &Input) -> io::Result<Self> {
        match input {
            Input::Stdin => Ok(Self::Stdin(BufReader::new(io::stdin()))),
            Input::File(path) => {
                let file = File::open(path).map_err(|e| {
                    io::Error::new(e.kind(), format!("Failed to read from: {}", path.display()))
                })?;

                Ok(Self::File(BufReader::new(file)))
            }
            Input::Mmap(mmap, _) => Ok(Self::Mmap(BufReader::new(MmapReader::new(mmap)))),
            Input::Bytes(bytes) => Ok(Self::Bytes(BufReader::new(BytesReader::new(bytes)))),
        }
    }
}

impl Read for InputReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            Self::Stdin(reader) => reader.read(buf),
            Self::File(reader) => reader.read(buf),
            Self::Mmap(reader) => reader.read(buf),
            Self::Bytes(reader) => reader.read(buf),
        }
    }
}

// Implement BufRead to allow direct use of lines() method
impl BufRead for InputReader {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        match self {
            Self::Stdin(reader) => reader.fill_buf(),
            Self::File(reader) => reader.fill_buf(),
            Self::Mmap(reader) => reader.fill_buf(),
            Self::Bytes(reader) => reader.fill_buf(),
        }
    }

    fn consume(&mut self, amt: usize) {
        match self {
            Self::Stdin(reader) => reader.consume(amt),
            Self::File(reader) => reader.consume(amt),
            Self::Mmap(reader) => reader.consume(amt),
            Self::Bytes(reader) => reader.consume(amt),
        }
    }
}

impl Input {
    /// Construct an `Input` from a file path or stdin (designated by "-").
    ///
    /// For bytes data, use `Input::from_bytes` instead.
    pub fn new<P: AsRef<Path>>(p: P, io: Io) -> Result<Self> {
        // Handle the stdin case
        let path_ref = p.as_ref();
        if path_ref.as_os_str() == "-" {
            return Ok(Self::Stdin);
        }

        match io {
            Io::Streamed | Io::Buffered => {
                let path_buf = path_ref.to_path_buf();

                Ok(Self::File(path_buf))
            }
            Io::MemoryMapped => {
                let path_buf = path_ref.to_path_buf();
                let file = File::open(&path_buf).with_context(|| {
                    format!(
                        "Failed to open file for memory mapping: {}",
                        path_buf.display()
                    )
                })?;

                // Safety: Memory mapping requires `unsafe` per memmap2 crate
                let mmap = unsafe { Mmap::map(&file)? };
                let mmap_arc = Arc::new(mmap);

                Ok(Self::Mmap(mmap_arc, path_buf))
            }
            Io::Bytes => {
                anyhow::bail!("For byte data with `Io::Bytes`, use `Input::from_bytes()`.")
            }
        }
    }

    /// Create an `Input` from byte data.
    pub fn from_bytes<B: AsRef<[u8]>>(bytes: B) -> Self {
        Self::Bytes(Arc::from(bytes.as_ref()))
    }

    /// A helper to get an `InputReader` from this input.
    pub fn reader(&self) -> io::Result<InputReader> {
        InputReader::new(self)
    }

    /// Returns the file name of the input or `"-"` for stdin.
    pub fn source(&self) -> String {
        match self {
            Self::Stdin => "-".to_string(),
            Self::File(path) | Self::Mmap(_, path) => path.file_name().map_or_else(
                || format!("No filename: {}", path.display()),
                |name| {
                    name.to_str().map_or_else(
                        || format!("Non-UTF-8 filename: {:?}", name),
                        |utf8_name| utf8_name.to_string(),
                    )
                },
            ),
            Self::Bytes(_) => "<bytes>".to_string(),
        }
    }

    /// Get the size of the input in bytes, if available.
    /// Returns `None` for stdin and when a filesize can't be determined.
    pub fn size(&self) -> Option<usize> {
        match self {
            Self::Stdin => None,
            Self::File(path) => fs::metadata(path)
                .map(|metadata| metadata.len() as usize)
                .ok(),
            Self::Mmap(mmap, _) => Some(mmap.len()),
            Self::Bytes(bytes) => Some(bytes.len()),
        }
    }
}
