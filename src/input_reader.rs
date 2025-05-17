//! Input readers implementing Read and BufRead traits.

use crate::input::Input;
use memmap2::Mmap;
use std::cmp;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};
use std::sync::Arc;

/// A wrapper that implements Read and BufRead for each input type
/// when `read()` is called multiple times with state tracked between calls
#[derive(Debug)]
pub enum InputReader {
    Stdin(BufReader<io::Stdin>),
    File(BufReader<File>),
    Mmap(MmapReader),
    Bytes(BytesReader),
}

impl InputReader {
    /// Create an `InputReader` instance from an `Input`
    pub fn new(input: &Input) -> io::Result<Self> {
        match input {
            Input::Stdin => Ok(Self::Stdin(BufReader::new(io::stdin()))),
            Input::File(path) => {
                let file = File::open(path).map_err(|e| {
                    io::Error::new(e.kind(), format!("failed to read from: {}", path.display()))
                })?;

                Ok(Self::File(BufReader::new(file)))
            }
            Input::Mmap(mmap, _) => Ok(Self::Mmap(MmapReader::new(mmap))),
            Input::Bytes(bytes) => Ok(Self::Bytes(BytesReader::new(bytes))),
        }
    }
}

// The `impl BufRead` provides `lines()` for an `InputReader`
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

/// A reader that allows memory-mapped files to be used with the standard `Read` trait.
#[derive(Clone, Debug)]
pub struct MmapReader {
    mmap: Arc<Mmap>,
    position: usize,
}

impl MmapReader {
    pub fn new(mmap: &Arc<Mmap>) -> Self {
        Self {
            mmap: Arc::clone(mmap),
            position: 0,
        }
    }
}

impl BufRead for MmapReader {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        let remaining = self.mmap.len().saturating_sub(self.position);
        if remaining == 0 {
            return Ok(&[]);
        }

        Ok(&self.mmap[self.position..])
    }

    fn consume(&mut self, amt: usize) {
        self.position = cmp::min(self.position + amt, self.mmap.len());
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

/// A reader that provides a zero-copy for byte slices with the `Read` trait.
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

impl BufRead for BytesReader {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        let remaining = self.bytes.len().saturating_sub(self.position);
        if remaining == 0 {
            return Ok(&[]);
        }

        Ok(&self.bytes[self.position..])
    }

    fn consume(&mut self, amt: usize) {
        self.position = cmp::min(self.position + amt, self.bytes.len());
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
