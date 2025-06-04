//! A collection for tallying word counts using `HashMap`.

use std::{borrow::Cow, iter};

use hashbrown::{HashMap, hash_map};
use rustc_hash::FxBuildHasher;

use anyhow::{Context, Result};
use memchr::memchr2_iter;
use rayon::prelude::*;

use crate::options::{Options, case::Case, encoding::Encoding, io::Io, performance::Performance};
use crate::{Count, Input, Reader, Word, WordTallyError};

/// Map for tracking word counts with non-deterministic iteration order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TallyMap {
    inner: HashMap<Word, Count, FxBuildHasher>,
}

impl Default for TallyMap {
    fn default() -> Self {
        Self {
            inner: HashMap::with_hasher(FxBuildHasher),
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
            inner: HashMap::with_capacity_and_hasher(capacity, FxBuildHasher),
        }
    }

    /// Returns the number of unique words in the map.
    #[must_use]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns true if the map contains no words.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Returns an iterator over the counts.
    pub fn values(&self) -> impl Iterator<Item = &Count> {
        self.inner.values()
    }

    /// Reserves capacity for at least `additional` more elements.
    pub fn reserve(&mut self, additional: usize) {
        self.inner.reserve(additional);
    }

    /// Retains only the elements specified by the predicate.
    pub fn retain(&mut self, predicate: impl FnMut(&Word, &mut Count) -> bool) {
        self.inner.retain(predicate);
    }

    /// Extends the tally map with word counts from a string slice.
    ///
    /// # Errors
    ///
    /// Returns `Error::NonAsciiInAsciiMode` if `encoding` is ASCII and non-ASCII bytes are encountered.
    #[inline]
    pub fn add_words(&mut self, content: &str, case: Case, encoding: Encoding) -> Result<()> {
        encoding.segment_words(content, case, |word| match word {
            Cow::Borrowed(w) => self.increment(w),
            Cow::Owned(w) => self.increment_owned(w.into_boxed_str()),
        })
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

    /// Creates a `TallyMap` from an `Input` source and `Options`.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Input cannot be read (file not found, permission denied, etc.)
    /// - Input contains invalid UTF-8 data
    /// - A configured thread pool cannot be initialized
    /// - I/O errors occur during reading
    pub fn from_input(input: &Input, options: &Options) -> Result<Self> {
        match options.io() {
            Io::Stream => Self::stream_count(input, options),
            Io::ParallelStream => Self::par_stream_count(input, options),
            Io::ParallelInMemory => {
                let bytes = Self::read_input_to_bytes(input, options.performance())?;
                Self::par_memory_count(&bytes, options)
            }
            Io::ParallelBytes => match input {
                Input::View(view) => Self::par_memory_count(view.as_ref(), options),
                Input::Reader(_) => Err(WordTallyError::BytesInputRequired.into()),
            },
            Io::ParallelMmap => match input {
                Input::View(view) => Self::par_memory_count(view.as_ref(), options),
                Input::Reader(_) => Err(WordTallyError::MmapStdin.into()),
            },
        }
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

        Self::process_content_parallel(
            content,
            perf,
            options.case(),
            options.encoding(),
            Self::chunk_boundaries,
        )
    }

    /// Common logic for parallel content processing.
    fn process_content_parallel(
        content: &str,
        perf: &Performance,
        case: Case,
        encoding: Encoding,
        boundary_fn: fn(&str, usize) -> Vec<usize>,
    ) -> Result<Self> {
        let total_chunks = perf.total_chunks(content.len() as u64);
        let num_chunks =
            usize::try_from(total_chunks).map_err(|_| WordTallyError::ChunkCountExceeded {
                chunks: total_chunks,
            })?;
        let boundaries = boundary_fn(content, num_chunks);
        let tally = Self::process_chunks(content, &boundaries, case, encoding)?;

        Ok(tally)
    }

    /// Low-memory streamed word tallying.
    ///
    /// Processes data in chunks without loading the entire input into memory.
    /// Uses chunk-based processing for better performance than line-by-line.
    fn stream_count(input: &Input, options: &Options) -> Result<Self> {
        let perf = options.performance();
        let case = options.case();
        let encoding = options.encoding();

        // Get metadata before borrowing reader
        let source = input.source();
        let size = input.size();
        let (batch_size, target_batch_size) = Self::calculate_batch_sizes_from_size(size, perf)?;
        let tally_capacity = perf.chunk_capacity(size.unwrap_or_else(|| perf.base_stdin_size()));

        let Input::Reader(reader) = input else {
            anyhow::bail!("Stream mode requires a readable input")
        };

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
                        tally.add_words(text, case, encoding)?;
                        remainder.clear();
                    }
                    Err(e) => {
                        Self::handle_utf8_error(
                            e,
                            &mut tally,
                            case,
                            encoding,
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
                .with_context(|| format!("invalid UTF-8 in stream from: {}", input.source()))?;
            tally.add_words(final_text, case, encoding)?;
        }

        Ok(tally)
    }

    /// Parallel, streamed word tallying.
    ///
    /// Processes data in parallel with Rayon while streaming:
    /// - Uses memchr SIMD to find whitespace-aligned chunk boundaries
    /// - Stream in chunks, without loading the entire input into memory
    /// - Balances performance and memory usage
    fn par_stream_count(input: &Input, options: &Options) -> Result<Self> {
        let perf = options.performance();
        let case = options.case();
        let encoding = options.encoding();

        // Get metadata before borrowing reader
        let source = input.source();
        let size = input.size();
        let (batch_size, target_batch_size) = Self::calculate_batch_sizes_from_size(size, perf)?;
        let tally_capacity = perf.capacity(size);

        let Input::Reader(reader) = input else {
            anyhow::bail!("Parallel stream mode requires a readable input")
        };

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
                    encoding,
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
    fn process_chunks(
        content: &str,
        boundaries: &[usize],
        case: Case,
        encoding: Encoding,
    ) -> Result<Self> {
        boundaries
            .windows(2)
            .par_bridge()
            .filter_map(|window| {
                let (start, end) = (window[0], window[1]);
                (end > start).then(|| &content[start..end])
            })
            .map(|chunk| {
                let mut tally = Self::new();
                tally.add_words(chunk, case, encoding)?;
                Ok(tally)
            })
            .try_reduce(Self::new, |acc, tally| Ok(acc.merge(tally)))
    }

    /// Reads the entire input into a byte buffer.
    fn read_input_to_bytes(input: &Input, perf: &Performance) -> Result<Vec<u8>> {
        let mut buffer = Vec::with_capacity(perf.capacity(input.size()));

        match input {
            Input::Reader(reader) => {
                reader.with_buf_read(|buf_read| {
                    buf_read.read_to_end(&mut buffer).with_context(|| {
                        format!("failed to read input into buffer: {}", input.source())
                    })
                })?;
            }
            Input::View(view) => buffer.extend_from_slice(view.as_ref()),
        }

        Ok(buffer)
    }

    /// Calculate batch sizes from optional size.
    fn calculate_batch_sizes_from_size(
        size: Option<u64>,
        perf: &Performance,
    ) -> Result<(usize, usize)> {
        let stream_batch = Performance::stream_batch_size();
        let batch_size = usize::try_from(stream_batch)
            .map_err(|_| WordTallyError::BatchSizeExceeded { size: stream_batch })?;
        let input_size = size.unwrap_or_else(|| perf.base_stdin_size());
        let target_size = perf.stream_batch_size_for_input(input_size);
        let target_batch_size = usize::try_from(target_size)
            .map_err(|_| WordTallyError::BatchSizeExceeded { size: target_size })?;

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

        let mut boundaries = Vec::with_capacity(num_chunks + 1);
        boundaries.extend(iter::once(0).chain((1..=num_chunks).map(|i| {
            let target_pos = i * target_chunk_size;
            Self::find_whitespace_boundary(content_bytes, target_pos)
        })));
        boundaries
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
        reader: &Reader,
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
        })
    }

    /// Process the stream buffer and return tally update with bytes processed.
    ///
    /// Returns `Some((tally, bytes))` if data was processed, `None` if more data is needed.
    fn process_stream_buffer(
        buffer: &[u8],
        target_batch_size: usize,
        reached_eof: bool,
        case: Case,
        encoding: Encoding,
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
                tally.add_words(remaining, case, encoding)?;
                Ok(Some((tally, buffer.len())))
            }
            _ => {
                // Multiple chunks: process in parallel
                let process_end = chunk_boundaries[chunk_boundaries.len() - 1];
                let content = Self::parse_utf8_slice(&buffer[..process_end])?;
                let batch_result =
                    Self::process_chunks(content, &chunk_boundaries, case, encoding)?;
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
        encoding: Encoding,
        remainder: &mut Vec<u8>,
        buffer: &[u8],
        process_until: usize,
    ) -> Result<()> {
        let valid_up_to = e.valid_up_to();
        if valid_up_to > 0 {
            // simdutf8 guarantees this is valid UTF-8
            let valid = std::str::from_utf8(&buffer[..valid_up_to])
                .expect("simdutf8 guarantees valid UTF-8 up to valid_up_to");
            tally.add_words(valid, case, encoding)?;
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
