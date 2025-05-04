use anyhow::{Context, Result};
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::PathBuf;

/// `Input` to read from a file or stdin source.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Input {
    File(PathBuf),
    Stdin,
}

impl Default for Input {
    fn default() -> Self {
        Self::Stdin
    }
}

impl std::fmt::Display for Input {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::File(path) => write!(f, "File({})", path.display()),
            Self::Stdin => write!(f, "Stdin"),
        }
    }
}

impl Input {
    /// Construct an `Input` from a file path or stdin.
    pub fn new(path: &str) -> Result<Self> {
        if path == "-" {
            Ok(Self::Stdin)
        } else {
            Ok(Self::File(PathBuf::from(path)))
        }
    }

    /// Gets the reader from the input source.
    pub fn get_reader(&self, source: &str) -> Result<Box<dyn Read>> {
        match self {
            Self::File(path) => {
                let file =
                    File::open(path).with_context(|| format!("Failed to read from {}", source))?;
                Ok(Box::new(file))
            }
            Self::Stdin => Ok(Box::new(io::stdin())),
        }
    }

    /// Returns the file name of the input or `"-"` for stdin.
    pub fn source(&self) -> String {
        match self {
            Self::File(path) => path
                .file_name()
                .expect("File name inaccessible.")
                .to_str()
                .expect("File name invalid UTF-8.")
                .to_string(),
            Self::Stdin => "-".to_string(),
        }
    }

    /// Get the size of the input in bytes, if available.
    /// Returns None for stdin or if the file size can't be determined.
    pub fn size(&self) -> Option<u64> {
        match self {
            Self::File(path) => fs::metadata(path).map(|metadata| metadata.len()).ok(),
            Self::Stdin => None,
        }
    }
}
