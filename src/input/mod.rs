//! Input handling with buffered and mapped sources.
//!
//! This module provides two complementary input strategies:
//!
//! - **Buffered**:
//!   - Sequential access via buffered reading to stdin and files
//!   - Used by `Stream` and `ParallelStream` I/O modes
//!
//! - **Mapped**:
//!   - Zero-copy access to memory-mapped files and in-memory bytes
//!   - Used by `ParallelMmap`, `ParallelBytes` and `ParallelInMemory` I/O modes

pub mod buffered;
pub mod file_type;
pub mod mapped;

use std::{
    fs::{self, File},
    io,
    path::Path,
};

pub use self::{buffered::Buffered, file_type::FileType, mapped::Mapped};
use crate::WordTallyError;

/// Provides metadata for data sources.
pub trait Metadata {
    /// Returns the file path, if file-based.
    fn path(&self) -> Option<&Path>;

    /// Returns the size in bytes, if known.
    fn size(&self) -> Option<u64>;

    /// Returns the file type, if applicable.
    ///
    /// Returns `None` for stdin or in-memory data sources.
    fn file_type(&self) -> Option<FileType> {
        None
    }
}

/// Opens a file with enhanced error context.
///
/// # Errors
///
/// Returns `WordTallyError::Io` with specific messages for:
/// - File not found
/// - Permission denied
/// - Other I/O errors
pub(super) fn open_file_with_error_context(path: &Path) -> Result<File, WordTallyError> {
    File::open(path).map_err(|source| {
        let path_str = path.display().to_string();

        // Try to get file type if the file exists
        let file_type = fs::metadata(path).ok().map(|m| FileType::from(&m));

        let message = match source.kind() {
            io::ErrorKind::NotFound => "no such file".to_string(),
            io::ErrorKind::PermissionDenied => "permission denied".to_string(),
            io::ErrorKind::IsADirectory => "is a directory".to_string(),
            _ => "failed to open file".to_string(),
        };

        WordTallyError::Io {
            path: path_str,
            message,
            file_type,
            source,
        }
    })
}
