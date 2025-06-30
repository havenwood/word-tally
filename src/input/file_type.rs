//! File type enumeration for error reporting.

use std::{
    fmt::{self, Display, Formatter},
    fs,
};

use serde::{Deserialize, Serialize};

/// Represents different file types for error reporting.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum FileType {
    /// Regular file
    File,
    /// Directory
    Directory,
    /// Named pipe (FIFO)
    NamedPipe,
    /// Unix domain socket
    Socket,
    /// Character device (e.g., /dev/null)
    CharacterDevice,
    /// Block device (e.g., /dev/sda)
    BlockDevice,
    /// Other file types
    Special,
}

impl Display for FileType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::File => "file",
            Self::Directory => "directory",
            Self::NamedPipe => "named pipe",
            Self::Socket => "socket",
            Self::CharacterDevice => "character device",
            Self::BlockDevice => "block device",
            Self::Special => "special file",
        };
        f.write_str(s)
    }
}

impl From<&fs::Metadata> for FileType {
    #[cfg(unix)]
    fn from(metadata: &fs::Metadata) -> Self {
        use std::os::unix::fs::FileTypeExt;
        let file_type = metadata.file_type();

        if file_type.is_file() {
            Self::File
        } else if file_type.is_dir() {
            Self::Directory
        } else if file_type.is_fifo() {
            Self::NamedPipe
        } else if file_type.is_socket() {
            Self::Socket
        } else if file_type.is_char_device() {
            Self::CharacterDevice
        } else if file_type.is_block_device() {
            Self::BlockDevice
        } else {
            Self::Special
        }
    }

    #[cfg(not(unix))]
    fn from(metadata: &fs::Metadata) -> Self {
        if metadata.is_file() {
            Self::File
        } else if metadata.is_dir() {
            Self::Directory
        } else {
            Self::Special
        }
    }
}
