//! Word encoding strategies for text processing and validation.

use std::{
    borrow::Cow,
    fmt::{self, Display, Formatter},
};

use anyhow::Result;
use clap::ValueEnum;
use icu_segmenter::{WordSegmenter, options::WordBreakInvariantOptions};
use serde::{Deserialize, Serialize};

use crate::{error::Error, options::case::Case};

thread_local! {
    static WORD_SEGMENTER: WordSegmenter = WordSegmenter::new_dictionary(WordBreakInvariantOptions::default()).static_to_owned();
}

/// Determines how word boundaries are detected.
///
/// # Examples
///
/// ```
/// use word_tally::options::encoding::Encoding;
///
/// // Unicode: handles international text
/// // "café" → ["café"] (preserves accents)
/// // "naïve" → ["naïve"] (handles diacritics)
/// let unicode = Encoding::Unicode;
///
/// // ASCII: validates ASCII-only, rejects non-ASCII
/// // "café" → error (non-ASCII rejected)
/// // "naïve" → error (non-ASCII rejected)
/// let ascii = Encoding::Ascii;
/// ```
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    ValueEnum,
)]
#[serde(rename_all = "camelCase")]
pub enum Encoding {
    /// Unicode text encoding - accepts any UTF-8 text (default).
    #[default]
    #[value(alias = "utf8", alias = "utf-8")]
    Unicode,

    /// ASCII-only encoding - validates and rejects non-ASCII bytes.
    Ascii,
}

impl Display for Encoding {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unicode => write!(f, "unicode"),
            Self::Ascii => write!(f, "ascii"),
        }
    }
}

impl Encoding {
    /// Process text and segment into words based on this encoding.
    ///
    /// # Errors
    ///
    /// Returns `Error::NonAsciiInAsciiMode` if encoding is ASCII and non-ASCII bytes are
    /// encountered.
    #[inline]
    pub(crate) fn segment_words(
        self,
        content: &str,
        case: Case,
        mut word_fn: impl FnMut(Cow<'_, str>),
    ) -> Result<()> {
        match self {
            Self::Unicode => {
                WORD_SEGMENTER.with(|segmenter| {
                    let mut last_boundary = 0;
                    segmenter
                        .as_borrowed()
                        .segment_str(content)
                        .iter_with_word_type()
                        .for_each(|(boundary, word_type)| {
                            if word_type.is_word_like() {
                                let word = &content[last_boundary..boundary];
                                word_fn(case.normalize_unicode(word));
                            }
                            last_boundary = boundary;
                        });
                });

                Ok(())
            }
            Self::Ascii => {
                let bytes = content.as_bytes();

                // Validate entire content is ASCII
                if let Some(position) = bytes.iter().position(|&b| !b.is_ascii()) {
                    return Err(Error::NonAsciiInAsciiMode {
                        byte: bytes[position],
                        position,
                    }
                    .into());
                }

                let mut pos = 0;
                while pos < bytes.len() {
                    // Skip non-word characters
                    while pos < bytes.len() && !bytes[pos].is_ascii_alphanumeric() {
                        pos += 1;
                    }

                    if pos >= bytes.len() {
                        break;
                    }

                    let word_start = pos;

                    // Find end of word (alphanumeric or apostrophe)
                    while pos < bytes.len()
                        && (bytes[pos].is_ascii_alphanumeric() || bytes[pos] == b'\'')
                    {
                        pos += 1;
                    }

                    // We know this slice is valid UTF-8 because all bytes are ASCII
                    let word = &content[word_start..pos];
                    word_fn(case.normalize_ascii(word));
                }

                Ok(())
            }
        }
    }
}
