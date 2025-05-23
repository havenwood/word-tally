//! Input readers implementing Read and `BufRead` traits.

use crate::input::Input;
use memmap2::Mmap;
use std::cmp;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};

/// A wrapper that implements `Read` and `BufRead` for each input type.
#[derive(Debug)]
pub enum InputReader<'a> {
    Stdin(BufReader<io::Stdin>),
    File(BufReader<File>),
    Mmap(MmapReader<'a>),
    Bytes(BytesReader<'a>),
}

impl<'a> InputReader<'a> {
    /// Create an `InputReader` instance from an `Input`.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file cannot be opened (for file-based inputs)
    /// - Standard input is not accessible
    /// - Permission is denied when accessing a file
    /// - The file does not exist
    pub fn new(input: &'a Input) -> io::Result<Self> {
        match input {
            Input::Stdin => Ok(Self::Stdin(BufReader::new(io::stdin()))),
            Input::File(path) => {
                let file = File::open(path).map_err(|e| match e.kind() {
                    io::ErrorKind::NotFound => {
                        io::Error::new(e.kind(), format!("no such file: {}", path.display()))
                    }
                    io::ErrorKind::PermissionDenied => {
                        io::Error::new(e.kind(), format!("permission denied: {}", path.display()))
                    }
                    _ => io::Error::new(
                        e.kind(),
                        format!("failed to open file: {} ({})", path.display(), e),
                    ),
                })?;

                Ok(Self::File(BufReader::new(file)))
            }
            Input::Mmap(mmap, _) => Ok(Self::Mmap(MmapReader::new(mmap.as_ref()))),
            Input::Bytes(bytes) => Ok(Self::Bytes(BytesReader::new(bytes.as_ref()))),
        }
    }
}

// The `impl BufRead` provides `lines()` for an `InputReader`
impl BufRead for InputReader<'_> {
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

impl Read for InputReader<'_> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            Self::Stdin(reader) => reader.read(buf),
            Self::File(reader) => reader.read(buf),
            Self::Mmap(reader) => reader.read(buf),
            Self::Bytes(reader) => reader.read(buf),
        }
    }
}

/// A generic reader for slice-based sources.
#[derive(Clone, Debug)]
pub struct SliceReader<T> {
    source: T,
    len: usize,
    position: usize,
}

impl<T> SliceReader<T> {
    /// `BufReader`'s current buffer size.
    const BUFFER_SIZE: usize = 8192;

    /// Check the number of bytes remaining in the source.
    pub const fn remaining(&self) -> usize {
        self.len.saturating_sub(self.position)
    }

    /// Check if the reader has reached the end of the source.
    pub const fn is_exhausted(&self) -> bool {
        self.position >= self.len
    }

    /// Update the position after consuming bytes.
    fn consume(&mut self, amt: usize) {
        self.position = cmp::min(self.position + amt, self.len);
    }
}

impl<T: AsRef<[u8]>> SliceReader<T> {
    pub fn new(source: T) -> Self {
        let len = source.as_ref().len();

        Self {
            source,
            len,
            position: 0,
        }
    }

    /// Get the current buffer window.
    fn current_buffer(&self) -> &[u8] {
        let end = cmp::min(self.position + Self::BUFFER_SIZE, self.len);
        &self.source.as_ref()[self.position..end]
    }

    /// Get a slice of the source data from the current position.
    fn get_slice(&self, amt: usize) -> &[u8] {
        &self.source.as_ref()[self.position..][..amt]
    }

    /// Read data into the provided buffer, returning the number of bytes read.
    fn read_into(&mut self, buf: &mut [u8]) -> usize {
        let amt = self.remaining().min(buf.len());

        buf[..amt].copy_from_slice(self.get_slice(amt));
        self.position += amt;

        amt
    }
}

/// A buffered-for-streaming, zero-copy reader for slice-based sources.
impl<T: AsRef<[u8]>> BufRead for SliceReader<T> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        if self.is_exhausted() {
            return Ok(&[]);
        }

        Ok(self.current_buffer())
    }

    fn consume(&mut self, amt: usize) {
        self.consume(amt);
    }
}

/// Reads bytes by copying them from slice-based sources.
impl<T: AsRef<[u8]>> Read for SliceReader<T> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.is_exhausted() {
            return Ok(0);
        }

        Ok(self.read_into(buf))
    }
}

/// A streaming, buffered reader for memory maps.
pub type MmapReader<'a> = SliceReader<&'a Mmap>;

/// A streaming, buffered reader for byte slices.
pub type BytesReader<'a> = SliceReader<&'a [u8]>;
