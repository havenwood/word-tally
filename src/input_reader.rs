//! Input readers implementing Read and BufRead traits.

use crate::input::Input;
use memmap2::Mmap;
use std::cmp;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};

/// A wrapper that implements Read and BufRead for each input type
/// when `read()` is called multiple times with state tracked between calls
#[derive(Debug)]
pub enum InputReader<'a> {
    Stdin(BufReader<io::Stdin>),
    File(BufReader<File>),
    Mmap(MmapReader<'a>),
    Bytes(BytesReader<'a>),
}

impl<'a> InputReader<'a> {
    /// Create an `InputReader` instance from an `Input`
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
impl<'a> BufRead for InputReader<'a> {
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

impl<'a> Read for InputReader<'a> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            Self::Stdin(reader) => reader.read(buf),
            Self::File(reader) => reader.read(buf),
            Self::Mmap(reader) => reader.read(buf),
            Self::Bytes(reader) => reader.read(buf),
        }
    }
}

/// A reader that allows memory-mapped files to be used with the standard `Read` trait.
#[derive(Clone, Debug)]
pub struct MmapReader<'a> {
    mmap: &'a Mmap,
    position: usize,
}

impl<'a> MmapReader<'a> {
    pub const fn new(mmap: &'a Mmap) -> Self {
        Self { mmap, position: 0 }
    }
}

/// A `BufRead` implementation for zero-copy, 8KB-buffered reading
impl<'a> BufRead for MmapReader<'a> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        let remaining = self.mmap.len().saturating_sub(self.position);
        if remaining == 0 {
            return Ok(&[]);
        }

        const BUFFER_SIZE: usize = 8192;
        let end = cmp::min(self.position + BUFFER_SIZE, self.mmap.len());

        Ok(&self.mmap[self.position..end])
    }

    fn consume(&mut self, amt: usize) {
        self.position = cmp::min(self.position + amt, self.mmap.len());
    }
}

impl<'a> Read for MmapReader<'a> {
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

/// A reader that allows byte slices to be used with the standard `Read` trait.
#[derive(Clone, Debug)]
pub struct BytesReader<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl<'a> BytesReader<'a> {
    pub const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, position: 0 }
    }
}

/// A `BufRead` implementation for zero-copy, 8KB-buffered reading
impl<'a> BufRead for BytesReader<'a> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        let remaining = self.bytes.len().saturating_sub(self.position);
        if remaining == 0 {
            return Ok(&[]);
        }

        const BUFFER_SIZE: usize = 8192;
        let end = cmp::min(self.position + BUFFER_SIZE, self.bytes.len());

        Ok(&self.bytes[self.position..end])
    }

    fn consume(&mut self, amt: usize) {
        self.position = cmp::min(self.position + amt, self.bytes.len());
    }
}

impl<'a> Read for BytesReader<'a> {
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
