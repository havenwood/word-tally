use anyhow::{Context, Result};
use std::fs::File;
use std::io::{self, Read};
use std::path::PathBuf;

/// `Input` to read from a file or stdin source.
pub enum Input {
    File(PathBuf),
    Stdin,
}

impl Input {
    /// Construct an `Input` from a file path or stdin.
    pub fn from_args(path: String) -> Result<Self> {
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
                .expect("File name invalid UTF-8."),
            Self::Stdin => "-",
        }
        .to_string()
    }
}
