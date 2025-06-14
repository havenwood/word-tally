//! Memory view for direct data access (mmap or bytes).

use std::{
    fmt::{self, Display, Formatter},
    ops::Deref,
    path::{Path, PathBuf},
};

use memmap2::Mmap;

use crate::{Metadata, WordTallyError, reader::open_file_with_error_context};

/// Memory view for direct data access (mmap or bytes).
#[derive(Debug)]
pub enum View {
    /// Memory-mapped file view.
    Mmap { path: PathBuf, mmap: Mmap },
    /// In-memory byte data.
    Bytes(Box<[u8]>),
}

impl AsRef<[u8]> for View {
    /// Returns the underlying byte slice.
    fn as_ref(&self) -> &[u8] {
        match self {
            Self::Mmap { mmap, .. } => mmap,
            Self::Bytes(bytes) => bytes,
        }
    }
}

impl Deref for View {
    type Target = [u8];

    /// Provides direct access to the underlying byte data.
    fn deref(&self) -> &Self::Target {
        match self {
            Self::Mmap { mmap, .. } => mmap,
            Self::Bytes(bytes) => bytes,
        }
    }
}

impl Display for View {
    /// Formats the view for display.
    /// Shows file path for mmap or "<bytes>" for byte data.
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Mmap { path, .. } => write!(f, "{}", path.display()),
            Self::Bytes(_) => write!(f, "<bytes>"),
        }
    }
}

impl From<Box<[u8]>> for View {
    /// Creates a view from boxed bytes.
    fn from(bytes: Box<[u8]>) -> Self {
        Self::Bytes(bytes)
    }
}

impl From<&[u8]> for View {
    /// Creates a view from a byte slice.
    fn from(bytes: &[u8]) -> Self {
        Self::Bytes(Box::from(bytes))
    }
}

impl From<Vec<u8>> for View {
    /// Creates a view from a byte vector.
    fn from(bytes: Vec<u8>) -> Self {
        Self::Bytes(bytes.into_boxed_slice())
    }
}

impl<const N: usize> From<&[u8; N]> for View {
    /// Creates a view from a fixed-size byte array.
    fn from(bytes: &[u8; N]) -> Self {
        Self::Bytes(Box::from(bytes.as_slice()))
    }
}

impl TryFrom<&Path> for View {
    type Error = WordTallyError;

    /// Creates a memory-mapped view from a file path.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - `WordTallyError::StdinInvalid` if path is "-" (stdin cannot be memory-mapped)
    /// - `WordTallyError::Io` if file cannot be opened (with specific messages for not found,
    ///   permission denied, etc.)
    /// - `WordTallyError::Io` if memory mapping fails
    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        if path.as_os_str() == "-" {
            return Err(WordTallyError::StdinInvalid);
        }
        let file = open_file_with_error_context(path)?;
        let path_buf = path.to_path_buf();

        // Safety: Memory mapping requires `unsafe` per memmap2 crate
        #[allow(unsafe_code)]
        let mmap = unsafe { Mmap::map(&file) }.map_err(|e| WordTallyError::Io {
            path: path_buf.display().to_string(),
            message: "failed to create memory map".into(),
            source: e,
        })?;

        Ok(Self::Mmap {
            path: path_buf,
            mmap,
        })
    }
}

impl TryFrom<PathBuf> for View {
    type Error = WordTallyError;

    /// Creates a memory-mapped view from a path buffer.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - `WordTallyError::StdinInvalid` if path is "-"
    /// - `WordTallyError::Io` if file cannot be opened or memory mapping fails
    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
        Self::try_from(path.as_path())
    }
}

impl TryFrom<&str> for View {
    type Error = WordTallyError;

    /// Creates a memory-mapped view from a string path.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - `WordTallyError::StdinInvalid` if path is "-"
    /// - `WordTallyError::Io` if file cannot be opened or memory mapping fails
    fn try_from(path: &str) -> Result<Self, Self::Error> {
        Self::try_from(Path::new(path))
    }
}

impl Metadata for View {
    /// Returns the file path for mmap views, `None` for bytes.
    fn path(&self) -> Option<&Path> {
        match self {
            Self::Mmap { path, .. } => Some(path),
            Self::Bytes(_) => None,
        }
    }

    /// Returns the view size in bytes.
    fn size(&self) -> Option<u64> {
        Some(self.len() as u64)
    }
}
