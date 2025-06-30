//! Mapped source for direct data access (mmap or bytes).

use std::{
    fmt::{self, Display, Formatter},
    ops::Deref,
    path::{Path, PathBuf},
};

use memmap2::Mmap;

use super::{FileType, Metadata, open_file_with_error_context};
use crate::WordTallyError;

/// Mapped source for direct data access (mmap or bytes).
#[derive(Debug)]
pub enum Mapped {
    /// Memory-mapped file with metadata.
    File {
        path: PathBuf,
        mmap: Mmap,
        file_type: FileType,
    },
    /// In-memory byte data.
    Memory { bytes: Box<[u8]> },
}

impl AsRef<[u8]> for Mapped {
    /// Returns the underlying byte slice.
    fn as_ref(&self) -> &[u8] {
        match self {
            Self::File { mmap, .. } => mmap,
            Self::Memory { bytes } => bytes,
        }
    }
}

impl Deref for Mapped {
    type Target = [u8];

    /// Provides direct access to the underlying byte data.
    fn deref(&self) -> &Self::Target {
        match self {
            Self::File { mmap, .. } => mmap,
            Self::Memory { bytes } => bytes,
        }
    }
}

impl Display for Mapped {
    /// Formats the view for display.
    /// Shows file path for file or "<bytes>" for memory data.
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::File { path, .. } => write!(f, "{}", path.display()),
            Self::Memory { .. } => write!(f, "<bytes>"),
        }
    }
}

impl From<Box<[u8]>> for Mapped {
    /// Creates a view from boxed bytes.
    fn from(bytes: Box<[u8]>) -> Self {
        Self::Memory { bytes }
    }
}

impl From<&[u8]> for Mapped {
    /// Creates a view from a byte slice.
    fn from(bytes: &[u8]) -> Self {
        Self::Memory {
            bytes: Box::from(bytes),
        }
    }
}

impl From<Vec<u8>> for Mapped {
    /// Creates a view from a byte vector.
    fn from(bytes: Vec<u8>) -> Self {
        Self::Memory {
            bytes: bytes.into_boxed_slice(),
        }
    }
}

impl<const N: usize> From<&[u8; N]> for Mapped {
    /// Creates a view from a fixed-size byte array.
    fn from(bytes: &[u8; N]) -> Self {
        Self::Memory {
            bytes: Box::from(bytes.as_slice()),
        }
    }
}

impl TryFrom<&Path> for Mapped {
    type Error = WordTallyError;

    /// Creates a memory-mapped view from a file path.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - `WordTallyError::Io` if file metadata cannot be read
    /// - `WordTallyError::NotMappable` if file type does not support memory mapping
    /// - `WordTallyError::Io` if file cannot be opened or memory mapping fails
    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        // Get metadata to check if file is mappable
        let metadata = std::fs::metadata(path).map_err(|source| {
            let path_str = path.display().to_string();
            let message = match source.kind() {
                std::io::ErrorKind::NotFound => "no such file".to_string(),
                std::io::ErrorKind::PermissionDenied => "permission denied".to_string(),
                _ => "failed to read metadata".to_string(),
            };
            WordTallyError::Io {
                path: path_str,
                message,
                file_type: None, // We don't have metadata yet
                source,
            }
        })?;

        // Get the file type from metadata
        let file_type = FileType::from(&metadata);

        // Check if file type is mappable (regular file or block device)
        #[cfg(unix)]
        let is_mappable = {
            use std::os::unix::fs::FileTypeExt;
            let ft = metadata.file_type();
            ft.is_file() || ft.is_block_device()
        };
        #[cfg(not(unix))]
        let is_mappable = metadata.is_file();

        if !is_mappable {
            return Err(WordTallyError::NotMappable {
                file_type,
                path: path.display().to_string(),
            });
        }

        // Now safe to open the file
        let file = open_file_with_error_context(path)?;

        // Safety: Memory mapping requires `unsafe` per memmap2 crate
        #[allow(unsafe_code)]
        let mmap = unsafe { Mmap::map(&file) }.map_err(|e| WordTallyError::Io {
            path: path.display().to_string(),
            message: "failed to create memory map".into(),
            file_type: Some(file_type),
            source: e,
        })?;

        Ok(Self::File {
            path: path.to_path_buf(),
            mmap,
            file_type,
        })
    }
}

impl TryFrom<PathBuf> for Mapped {
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

impl TryFrom<&str> for Mapped {
    type Error = WordTallyError;

    /// Creates a memory-mapped view from a string path.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - `WordTallyError::StdinNotMappable` if path is "-" (stdin cannot be memory-mapped)
    /// - `WordTallyError::NotMappable` if file type does not support memory mapping
    /// - `WordTallyError::Io` if file cannot be opened or memory mapping fails
    fn try_from(path: &str) -> Result<Self, Self::Error> {
        if path == "-" {
            return Err(WordTallyError::StdinNotMappable);
        }
        Self::try_from(Path::new(path))
    }
}

impl Metadata for Mapped {
    /// Returns the file path for file views, `None` for memory.
    fn path(&self) -> Option<&Path> {
        match self {
            Self::File { path, .. } => Some(path.as_path()),
            Self::Memory { .. } => None,
        }
    }

    /// Returns the size in bytes (always available).
    fn size(&self) -> Option<u64> {
        Some(self.len() as u64)
    }

    /// Returns the file type for file views, `None` for memory.
    fn file_type(&self) -> Option<FileType> {
        match self {
            Self::File { file_type, .. } => Some(*file_type),
            Self::Memory { .. } => None,
        }
    }
}
