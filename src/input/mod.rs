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
pub mod mapped;

use std::{fs::File, io, path::Path};

pub use self::{buffered::Buffered, mapped::Mapped};
use crate::WordTallyError;

/// Provides metadata for data sources.
pub trait Metadata {
    /// Returns the file path, if file-based.
    fn path(&self) -> Option<&Path>;

    /// Returns the size in bytes, if known.
    fn size(&self) -> Option<u64>;
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
        let path_buf = path.to_path_buf();
        let message = match source.kind() {
            io::ErrorKind::NotFound => format!("no such file: {}", path_buf.display()),
            io::ErrorKind::PermissionDenied => {
                format!("permission denied: {}", path_buf.display())
            }
            _ => format!("failed to open file: {}", path_buf.display()),
        };

        WordTallyError::Io {
            path: path_buf.display().to_string(),
            message,
            source,
        }
    })
}
