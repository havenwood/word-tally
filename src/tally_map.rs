//! A collection for tallying word counts using `HashMap`.

use std::{
    borrow::Cow,
    fs,
    io::{self, Read},
    iter,
    ops::{Deref, DerefMut},
    str,
};

use anyhow::{Context, Result};
use hashbrown::{HashMap, hash_map};
use icu_segmenter::{WordSegmenter, options::WordBreakInvariantOptions};
use memchr::memchr2_iter;
use rayon::prelude::*;

thread_local! {
    static WORD_SEGMENTER: WordSegmenter = WordSegmenter::new_dictionary(WordBreakInvariantOptions::default()).static_to_owned();
}

use crate::{
    Count, Metadata, Word, WordTallyError,
    input::{Buffered, Mapped},
    options::{Options, case::Case, io::Io, performance::Performance},
};

/// Map for tracking word counts with non-deterministic iteration order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TallyMap {
    inner: HashMap<Word, Count>,
}

impl Default for TallyMap {
    fn default() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }
}

impl TallyMap {
    /// Creates a new empty `TallyMap`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new `TallyMap` with the specified capacity.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: HashMap::with_capacity(capacity),
        }
    }

    /// Creates a `TallyMap` from a source path and `Options`.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file cannot be opened (file not found, permission denied, etc.)
    /// - The source is "-" but stdin cannot be read
    /// - The input contains invalid UTF-8 data
    /// - I/O errors occur during reading
    /// - `Io::ParallelBytes` is used (requires bytes input)
    /// - `Io::ParallelMmap` is used with stdin (memory mapping not supported)
    pub fn from_buffered_input(source: &str, options: &Options) -> Result<Self> {
        let reader = Buffered::try_from(source)?;
        Self::from_buffered(&reader, options)
    }

    /// Creates a `TallyMap` from a `Buffered` and `Options`.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The input contains invalid UTF-8 data
    /// - I/O errors occur during reading
    /// - `Io::ParallelBytes` is used (requires bytes input)
    /// - `Io::ParallelMmap` is used with stdin (memory mapping not supported)
    pub fn from_buffered(reader: &Buffered, options: &Options) -> Result<Self> {
        match options.io() {
            Io::Stream => Self::stream_count_reader(reader, options),
            Io::ParallelStream => Self::par_stream_count_reader(reader, options),
            Io::ParallelInMemory => {
                let bytes = Self::read_to_bytes(reader, options.performance())?;
                let view = Mapped::from(bytes);
                Self::from_mapped(&view, options)
            }
            Io::ParallelBytes => Err(WordTallyError::BytesRequired.into()),
            Io::ParallelMmap => Err(WordTallyError::StdinInvalid.into()),
        }
    }

    /// Creates a `TallyMap` from a source path and `Options`.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file cannot be read (file not found, permission denied, etc.)
    /// - The file cannot be memory-mapped
    /// - The input contains invalid UTF-8 data
    /// - `Io::Stream` or `Io::ParallelStream` is used (requires buffered input)
    pub fn from_mapped_input(source: &str, options: &Options) -> Result<Self> {
        let view = Mapped::try_from(source)?;
        Self::from_mapped(&view, options)
    }

    /// Creates a `TallyMap` from a `Mapped` and `Options`.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The input contains invalid UTF-8 data
    /// - `Io::Stream` or `Io::ParallelStream` is used (requires buffered input)
    pub fn from_mapped(view: &Mapped, options: &Options) -> Result<Self> {
        match options.io() {
            Io::Stream | Io::ParallelStream => Err(WordTallyError::Config(
                "stream mode requires a Buffered, not a Mapped".to_string(),
            )
            .into()),
            Io::ParallelInMemory | Io::ParallelBytes | Io::ParallelMmap => {
                Self::par_memory_count(view, options)
            }
        }
    }

    /// Creates a `TallyMap` from bytes and `Options`.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The input contains invalid UTF-8 data
    /// - `Io::Stream` or `Io::ParallelStream` is used (requires buffered input)
    pub fn from_bytes(bytes: Vec<u8>, options: &Options) -> Result<Self> {
        let view = Mapped::from(bytes);
        Self::from_mapped(&view, options)
    }

    /// Creates a `TallyMap` from a source path with bytes input handling.
    ///
    /// Handles the special case where "-" means stdin, reading the entire
    /// input into memory as bytes.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file cannot be read (file not found, permission denied, etc.)
    /// - I/O errors occur while reading stdin
    /// - The input contains invalid UTF-8 data
    /// - `Io::Stream` or `Io::ParallelStream` is used (requires buffered input)
    pub fn from_bytes_input(source: &str, options: &Options) -> Result<Self> {
        let bytes = if source == "-" {
            let mut buffer = Vec::new();
            io::stdin().lock().read_to_end(&mut buffer)?;
            buffer
        } else {
            fs::read(source)?
        };
        Self::from_bytes(bytes, options)
    }

    /// Extends the tally map with word counts from a string slice.
    #[inline]
    pub fn add_words(&mut self, content: &str, case: Case) {
        WORD_SEGMENTER.with(|segmenter| {
            let mut last_boundary = 0;
            segmenter
                .as_borrowed()
                .segment_str(content)
                .iter_with_word_type()
                .for_each(|(boundary, word_type)| {
                    if word_type.is_word_like() {
                        let word = &content[last_boundary..boundary];
                        match case.normalize_unicode(word) {
                            Cow::Borrowed(w) => self.increment(w),
                            Cow::Owned(w) => self.increment_owned(w.into_boxed_str()),
                        }
                    }
                    last_boundary = boundary;
                });
        });
    }

    /// Merge two tally maps, always merging the smaller into the larger.
    ///
    /// Returns the merged map containing the combined counts from both input maps.
    #[must_use]
    pub fn merge(mut self, mut other: Self) -> Self {
        // Always merge the smaller map into the larger for better performance
        if self.len() < other.len() {
            other.extend(self.inner.drain());
            other
        } else {
            self.extend(other.inner.drain());
            self
        }
    }

    /// Increments a word's tally by reference to avoid duplicate allocation.
    #[inline]
    fn increment(&mut self, word: &str) {
        match self.inner.entry_ref(word) {
            hash_map::EntryRef::Vacant(entry) => {
                entry.insert(1);
            }
            hash_map::EntryRef::Occupied(mut entry) => {
                *entry.get_mut() += 1;
            }
        }
    }

    /// Increments a word's tally using an owned key.
    #[inline]
    fn increment_owned(&mut self, word: Word) {
        *self.inner.entry(word).or_insert(0) += 1;
    }

    /// Parallel in-memory word tallying.
    ///
    /// Processes byte data in parallel with Rayon using direct memory access:
    /// - Works with both memory-mapped files and byte arrays
    /// - Validates UTF-8 once upfront using SIMD
    /// - Uses efficient backwards search for chunk boundaries
    fn par_memory_count(bytes: &[u8], options: &Options) -> Result<Self> {
        let perf = options.performance();
        let content = Self::parse_utf8_slice(bytes)?;

        Self::process_content_parallel(content, perf, options.case(), Self::chunk_boundaries)
    }

    /// Common logic for parallel content processing.
    fn process_content_parallel(
        content: &str,
        perf: &Performance,
        case: Case,
        boundary_fn: fn(&str, usize) -> Vec<usize>,
    ) -> Result<Self> {
        let total_chunks = perf.total_chunks(content.len() as u64);
        let num_chunks =
            usize::try_from(total_chunks).map_err(|_| WordTallyError::ChunkOverflow {
                chunks: total_chunks,
            })?;
        let boundaries = boundary_fn(content, num_chunks);
        let tally = Self::process_chunks(content, &boundaries, case)?;

        Ok(tally)
    }

    /// Low-memory streamed word tallying.
    ///
    /// Processes data in chunks without loading the entire input into memory.
    /// Uses chunk-based processing for better performance than line-by-line.
    fn stream_count_reader(reader: &Buffered, options: &Options) -> Result<Self> {
        let perf = options.performance();
        let case = options.case();

        // Get metadata before borrowing reader
        let source = reader.to_string();
        let size = reader.size();
        let (batch_size, target_batch_size) = Self::calculate_batch_sizes_from_size(size, perf)?;
        let tally_capacity = perf.chunk_capacity(size.unwrap_or_else(|| perf.base_stdin_size()));

        let mut tally = Self::with_capacity(tally_capacity);
        let mut buffer = Vec::with_capacity(batch_size);
        let mut remainder = Vec::new();
        let mut reached_eof = false;

        while !reached_eof || !buffer.is_empty() {
            // Fill buffer
            Self::fill_stream_buffer(
                reader,
                &mut buffer,
                &mut reached_eof,
                target_batch_size,
                &source,
            )?;

            // Find the last whitespace (space or newline) to ensure we don't split words
            let process_until = if reached_eof {
                buffer.len()
            } else {
                Self::find_whitespace_boundary(&buffer, buffer.len())
                    .min(buffer.len())
                    .max(0)
            };

            if process_until > 0 {
                // Process content
                let chunk = &buffer[..process_until];
                let content = if remainder.is_empty() {
                    simdutf8::compat::from_utf8(chunk)
                } else {
                    remainder.extend_from_slice(chunk);
                    simdutf8::compat::from_utf8(&remainder)
                };

                match content {
                    Ok(text) => {
                        tally.add_words(text, case);
                        remainder.clear();
                    }
                    Err(e) => {
                        Self::handle_utf8_error(
                            e,
                            &mut tally,
                            case,
                            &mut remainder,
                            &buffer,
                            process_until,
                        )?;
                    }
                }

                buffer.drain(..process_until);
            }
        }

        // Process any remaining data
        if !remainder.is_empty() {
            let final_text = simdutf8::basic::from_utf8(&remainder)
                .with_context(|| format!("invalid UTF-8 in stream from: {source}"))?;
            tally.add_words(final_text, case);
        }

        Ok(tally)
    }

    /// Parallel, streamed word tallying.
    ///
    /// Processes data in parallel with Rayon while streaming:
    /// - Uses memchr SIMD to find whitespace-aligned chunk boundaries
    /// - Stream in chunks, without loading the entire input into memory
    /// - Balances performance and memory usage
    fn par_stream_count_reader(reader: &Buffered, options: &Options) -> Result<Self> {
        let perf = options.performance();
        let case = options.case();

        // Get metadata before borrowing reader
        let source = reader.to_string();
        let size = reader.size();
        let (batch_size, target_batch_size) = Self::calculate_batch_sizes_from_size(size, perf)?;
        let tally_capacity = perf.capacity(size);

        let initial_tally = Self::with_capacity(tally_capacity);

        let final_tally = iter::from_fn({
            let mut buffer = Vec::with_capacity(batch_size);
            let mut reached_eof = false;

            move || {
                if reached_eof && buffer.is_empty() {
                    return None;
                }

                // Fill buffer
                if let Err(e) = Self::fill_stream_buffer(
                    reader,
                    &mut buffer,
                    &mut reached_eof,
                    target_batch_size,
                    &source,
                ) {
                    return Some(Err(e));
                }

                // Process buffer
                match Self::process_stream_buffer(
                    &buffer,
                    target_batch_size,
                    reached_eof,
                    case,
                    perf,
                ) {
                    Ok(Some((tally_update, processed_bytes))) => {
                        buffer.drain(..processed_bytes);
                        Some(Ok(tally_update))
                    }
                    Ok(None) if reached_eof => None,
                    Ok(None) => Some(Ok(Self::new())),
                    Err(e) => Some(Err(e)),
                }
            }
        })
        .try_fold(initial_tally, |acc, result| {
            result.map(|update| acc.merge(update))
        })?;

        Ok(final_tally)
    }

    /// Process content in parallel using provided chunk boundaries.
    fn process_chunks(content: &str, boundaries: &[usize], case: Case) -> Result<Self> {
        boundaries
            .windows(2)
            .par_bridge()
            .filter_map(|window| {
                let (start, end) = (window[0], window[1]);
                (end > start).then(|| &content[start..end])
            })
            .map(|chunk| {
                let mut tally = Self::new();
                tally.add_words(chunk, case);
                Ok(tally)
            })
            .try_reduce(Self::new, |acc, tally| Ok(acc.merge(tally)))
    }

    /// Reads the entire input into a byte buffer.
    fn read_to_bytes(reader: &Buffered, perf: &Performance) -> Result<Vec<u8>> {
        let mut buffer = Vec::with_capacity(perf.capacity(reader.size()));

        reader.with_buf_read(|buf_read| {
            buf_read
                .read_to_end(&mut buffer)
                .with_context(|| format!("failed to read input into buffer: {reader}"))
        })??;

        Ok(buffer)
    }

    /// Calculate batch sizes from optional size.
    fn calculate_batch_sizes_from_size(
        size: Option<u64>,
        perf: &Performance,
    ) -> Result<(usize, usize)> {
        let stream_batch = Performance::stream_batch_size();
        let batch_size = usize::try_from(stream_batch)
            .map_err(|_| WordTallyError::BatchOverflow { size: stream_batch })?;
        let input_size = size.unwrap_or_else(|| perf.base_stdin_size());
        let target_size = perf.stream_batch_size_for_input(input_size);
        let target_batch_size = usize::try_from(target_size)
            .map_err(|_| WordTallyError::BatchOverflow { size: target_size })?;

        Ok((batch_size, target_batch_size))
    }

    /// Find whitespace-aligned boundary at or before the target position.
    fn find_whitespace_boundary(content_bytes: &[u8], target_pos: usize) -> usize {
        if target_pos >= content_bytes.len() {
            return content_bytes.len();
        }
        let search_region = &content_bytes[..target_pos];
        memchr::memrchr2(b' ', b'\n', search_region).map_or(target_pos, |pos| pos + 1)
    }

    /// Find chunk boundaries using efficient backwards search from target positions.
    fn chunk_boundaries(content: &str, num_chunks: usize) -> Vec<usize> {
        let content_len = content.len();
        let target_chunk_size = content_len.div_ceil(num_chunks);
        let content_bytes = content.as_bytes();

        iter::once(0)
            .chain((1..=num_chunks).map(|i| {
                let target_pos = i * target_chunk_size;
                Self::find_whitespace_boundary(content_bytes, target_pos)
            }))
            .collect()
    }

    /// Calculate chunk boundaries within a streamed batch based on whitespace positions.
    fn streamed_boundaries(
        buffer: &[u8],
        whitespace_positions: &[usize],
        target_batch_size: usize,
        reached_eof: bool,
    ) -> Vec<usize> {
        let chunk_size = target_batch_size / Performance::PAR_CHUNKS_PER_THREAD as usize;
        let mut boundaries =
            Self::calculate_boundaries_from_positions(whitespace_positions, chunk_size, 0);

        // Add the end of buffer at end of file if needed
        if reached_eof && boundaries.last() < Some(&buffer.len()) {
            boundaries.push(buffer.len());
        }

        boundaries
    }

    /// Calculate boundaries from positions based on minimum chunk size.
    fn calculate_boundaries_from_positions(
        positions: &[usize],
        min_chunk_size: usize,
        start_pos: usize,
    ) -> Vec<usize> {
        iter::once(start_pos)
            .chain(
                positions
                    .iter()
                    .scan(start_pos, |last_boundary, &pos| {
                        if pos - *last_boundary >= min_chunk_size {
                            *last_boundary = pos + 1;
                            Some(Some(pos + 1))
                        } else {
                            Some(None)
                        }
                    })
                    .flatten(),
            )
            .collect()
    }

    /// Fill the buffer with data from the stream reader up to the target size.
    fn fill_stream_buffer(
        reader: &Buffered,
        buffer: &mut Vec<u8>,
        reached_eof: &mut bool,
        target_size: usize,
        source: &str,
    ) -> Result<()> {
        reader.with_buf_read(|buf_read| {
            while buffer.len() < target_size && !*reached_eof {
                // Fill reader's internal buffer
                let available = buf_read
                    .fill_buf()
                    .with_context(|| format!("failed to read from: {source}"))?;

                if available.is_empty() {
                    *reached_eof = true;
                    break;
                }

                // Copy only what's needed to reach target size
                let bytes_to_copy = available.len().min(target_size - buffer.len());
                buffer.extend_from_slice(&available[..bytes_to_copy]);
                // Mark bytes as consumed in reader
                buf_read.consume(bytes_to_copy);
            }
            Ok(())
        })?
    }

    /// Process the stream buffer and return tally update with bytes processed.
    ///
    /// Returns `Some((tally, bytes))` if data was processed, `None` if more data is needed.
    fn process_stream_buffer(
        buffer: &[u8],
        target_batch_size: usize,
        reached_eof: bool,
        case: Case,
        perf: &Performance,
    ) -> Result<Option<(Self, usize)>> {
        // Find all whitespace positions with SIMD search
        let whitespace_positions: Vec<usize> = memchr2_iter(b' ', b'\n', buffer).collect();

        // Wait for more data if buffer is incomplete and not at EOF
        if whitespace_positions.is_empty() && !reached_eof && buffer.len() < target_batch_size {
            return Ok(None);
        }

        // Calculate chunk boundaries aligned to whitespace for parallel processing
        let chunk_boundaries = Self::streamed_boundaries(
            buffer,
            &whitespace_positions,
            target_batch_size,
            reached_eof,
        );

        match chunk_boundaries.len() {
            0 => Ok(None),
            1 if reached_eof && !buffer.is_empty() => {
                // Final chunk: process sequentially
                let remaining = Self::parse_utf8_slice(buffer)?;
                let mut tally = Self::with_capacity(perf.chunk_capacity(buffer.len() as u64));
                tally.add_words(remaining, case);
                Ok(Some((tally, buffer.len())))
            }
            _ => {
                // Multiple chunks: process in parallel
                let process_end = chunk_boundaries[chunk_boundaries.len() - 1];
                let content = Self::parse_utf8_slice(&buffer[..process_end])?;
                let batch_result = Self::process_chunks(content, &chunk_boundaries, case)?;
                Ok(Some((batch_result, process_end)))
            }
        }
    }

    /// Parse a UTF-8 slice with fast validation.
    #[inline]
    fn parse_utf8_slice(buffer: &[u8]) -> Result<&str> {
        simdutf8::basic::from_utf8(buffer).map_err(|_| {
            WordTallyError::Utf8 {
                byte: 0,
                message: "invalid UTF-8 sequence".into(),
            }
            .into()
        })
    }

    /// Handle UTF-8 decoding errors by processing valid portion and preserving invalid bytes.
    fn handle_utf8_error(
        e: simdutf8::compat::Utf8Error,
        tally: &mut Self,
        case: Case,
        remainder: &mut Vec<u8>,
        buffer: &[u8],
        process_until: usize,
    ) -> Result<()> {
        let valid_up_to = e.valid_up_to();
        if valid_up_to > 0 {
            // simdutf8 guarantees this is valid UTF-8
            let valid =
                str::from_utf8(&buffer[..valid_up_to]).map_err(|_| WordTallyError::Utf8 {
                    byte: valid_up_to,
                    message: "simdutf8 validation inconsistency".to_string(),
                })?;
            tally.add_words(valid, case);
        }
        // Keep invalid portion for next iteration
        remainder.clear();
        remainder.extend_from_slice(&buffer[valid_up_to..process_until]);
        Ok(())
    }
}

impl IntoIterator for TallyMap {
    type Item = (Word, Count);
    type IntoIter = hash_map::IntoIter<Word, Count>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl FromIterator<(Word, Count)> for TallyMap {
    fn from_iter<I: IntoIterator<Item = (Word, Count)>>(iter: I) -> Self {
        let iter = iter.into_iter();
        let (size_hint, _) = iter.size_hint();
        let mut map = Self::with_capacity(size_hint);
        map.extend(iter);
        map
    }
}

impl Extend<(Word, Count)> for TallyMap {
    fn extend<T: IntoIterator<Item = (Word, Count)>>(&mut self, tally: T) {
        let iter = tally.into_iter();
        let (lower_bound, _) = iter.size_hint();

        self.inner.reserve(lower_bound);
        iter.for_each(|(word, count)| match self.inner.entry_ref(word.as_ref()) {
            hash_map::EntryRef::Occupied(mut entry) => {
                *entry.get_mut() += count;
            }
            hash_map::EntryRef::Vacant(_) => {
                self.inner.insert(word, count);
            }
        });
    }
}

impl Deref for TallyMap {
    type Target = HashMap<Word, Count>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for TallyMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
