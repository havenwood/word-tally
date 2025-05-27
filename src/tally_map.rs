//! A collection for tallying word counts using `HashMap`.

use std::{
    io::{BufRead, Read},
    iter, str,
    sync::Arc,
};

use hashbrown::{HashMap, hash_map};

use anyhow::{Context, Result};
use icu_segmenter::{WordSegmenter, options::WordBreakInvariantOptions};
use memchr::memchr2_iter;
use memmap2::Mmap;
use rayon::prelude::*;

use crate::options::{
    Options,
    case::Case::{self, Lower, Original, Upper},
    io::Io,
    performance::Performance,
    processing::Processing,
};
use crate::{Count, Input, Word, WordTallyError};

thread_local! {
    static WORD_SEGMENTER: WordSegmenter = WordSegmenter::new_dictionary(WordBreakInvariantOptions::default()).static_to_owned();
}

/// Map for tracking word counts with non-deterministic iteration order.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TallyMap {
    inner: HashMap<Word, Count>,
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
    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&Word, &mut Count) -> bool,
    {
        self.inner.retain(f);
    }

    /// Extends the tally map with word counts from a string slice.
    #[inline]
    pub fn add_words_from(&mut self, content: &str, case: Case) {
        // Use thread-local ICU segmenter with word types to identify actual words
        WORD_SEGMENTER.with(|segmenter| {
            let mut last_boundary = 0;
            segmenter
                .as_borrowed()
                .segment_str(content)
                .iter_with_word_type()
                .for_each(|(boundary, word_type)| {
                    if word_type.is_word_like() {
                        let word = &content[last_boundary..boundary];
                        match case {
                            // Avoid duplicate allocation
                            Original => self.increment(word),
                            Lower => {
                                if word.chars().all(|c| !c.is_uppercase()) {
                                    // Also avoid duplicate allocation when already all lowercase
                                    self.increment(word);
                                } else {
                                    self.increment_owned(word.to_lowercase().into_boxed_str());
                                }
                            }
                            Upper => self.increment_owned(word.to_uppercase().into_boxed_str()),
                        }
                    }
                    last_boundary = boundary;
                });
        });
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
        match (options.processing(), options.io()) {
            // Sequential processing
            (Processing::Sequential, Io::Streamed | Io::MemoryMapped) => {
                Self::streamed_count(input, options)
            }
            (Processing::Sequential, Io::Buffered | Io::Bytes) => {
                Self::buffered_count(input, options)
            }

            // Parallel processing
            (Processing::Parallel, Io::MemoryMapped) => {
                let Input::Mmap(mmap_arc, _) = input else {
                    return Err(WordTallyError::MmapStdin.into());
                };
                Self::par_mmap_count(mmap_arc, options)
            }
            (Processing::Parallel, Io::Streamed) => Self::par_streamed_count(input, options),
            (Processing::Parallel, Io::Buffered | Io::Bytes) => {
                Self::par_buffered_count(input, options)
            }
        }
    }

    /// Sequential buffered word tallying.
    ///
    /// Reads the entire input into memory before processing sequentially.
    fn buffered_count(input: &Input, options: &Options) -> Result<Self> {
        let perf = options.performance();
        let content = Self::read_input_to_string(input, perf)?;
        let mut tally = Self::with_capacity(perf.capacity(input.size()));
        tally.add_words_from(&content, options.case());

        Ok(tally)
    }

    /// Parallel buffered word tallying.
    ///
    /// Processes data in parallel with Rayon after loading entire content:
    /// - Uses memchr SIMD to find newline-aligned chunk boundaries
    /// - Trades higher memory usage for processing speed
    fn par_buffered_count(input: &Input, options: &Options) -> Result<Self> {
        let perf = options.performance();
        let content = Self::read_input_to_string(input, perf)?;

        Self::process_content_parallel(&content, perf, options.case(), Self::chunk_boundaries)
    }

    /// Parallel memory-mapped word tallying.
    ///
    /// Processes data in parallel with Rayon using memory-mapped slicing of files:
    /// - Uses memmap2 to access file contents without loading the entire file into memory
    /// - Optimized chunk boundary detection using SIMD newline scanning
    /// - Provides high performance with moderate memory usage
    ///
    /// # Limitations
    ///
    /// Memory mapping only works with seekable files (regular files on disk).
    /// It will not work with stdin, pipes, or other non-seekable sources.
    fn par_mmap_count(mmap: &Arc<Mmap>, options: &Options) -> Result<Self> {
        let perf = options.performance();
        let content = Self::parse_utf8_slice(mmap)?;

        Self::process_content_parallel(content, perf, options.case(), Self::mmap_boundaries)
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
            usize::try_from(total_chunks).map_err(|_| WordTallyError::ChunkCountExceeded {
                chunks: total_chunks,
            })?;
        let boundaries = boundary_fn(content, num_chunks);
        let tally = Self::process_chunks(content, &boundaries, case);

        Ok(tally)
    }

    /// Sequential streamed word tallying.
    ///
    /// Processes data in chunks without loading the entire input into memory.
    /// Uses chunk-based processing for better performance than line-by-line.
    fn streamed_count(input: &Input, options: &Options) -> Result<Self> {
        let perf = options.performance();
        let case = options.case();
        let mut reader = input.reader()?;

        let (batch_size, target_batch_size) = Self::calculate_batch_sizes(input, perf)?;
        let mut tally = Self::with_capacity(perf.capacity(input.size()));
        let mut buffer = Vec::with_capacity(batch_size);
        let mut remainder = Vec::new();
        let mut reached_eof = false;

        while !reached_eof || !buffer.is_empty() {
            // Fill buffer
            Self::fill_stream_buffer(
                &mut reader,
                &mut buffer,
                &mut reached_eof,
                target_batch_size,
                input,
            )?;

            // Find the last whitespace (space or newline) to ensure we don't split words
            let process_until = if reached_eof {
                buffer.len()
            } else {
                memchr::memrchr2(b' ', b'\n', &buffer).map_or(0, |pos| pos + 1)
            };

            if process_until > 0 {
                // Process content
                let chunk = &buffer[..process_until];
                let content = if remainder.is_empty() {
                    str::from_utf8(chunk)
                } else {
                    remainder.extend_from_slice(chunk);
                    str::from_utf8(&remainder)
                };

                match content {
                    Ok(text) => {
                        tally.add_words_from(text, case);
                        remainder.clear();
                    }
                    Err(e) => {
                        // Handle UTF-8 boundary case
                        if e.valid_up_to() > 0 {
                            let valid =
                                str::from_utf8(&buffer[..e.valid_up_to()]).expect("valid UTF-8");
                            tally.add_words_from(valid, case);
                        }
                        // Keep invalid portion for next iteration
                        remainder.clear();
                        remainder.extend_from_slice(&buffer[e.valid_up_to()..process_until]);
                    }
                }

                buffer.drain(..process_until);
            }
        }

        // Process any remaining data
        if !remainder.is_empty() {
            let final_text = str::from_utf8(&remainder)
                .with_context(|| format!("invalid UTF-8 in stream from: {}", input.source()))?;
            tally.add_words_from(final_text, case);
        }

        Ok(tally)
    }

    /// Parallel, streamed word tallying.
    ///
    /// Processes data in parallel with Rayon while streaming:
    /// - Uses memchr SIMD to find whitespace-aligned chunk boundaries
    /// - Stream in chunks, without loading the entire input into memory
    /// - Balances performance and memory usage
    fn par_streamed_count(input: &Input, options: &Options) -> Result<Self> {
        let perf = options.performance();
        let case = options.case();
        let mut reader = input.reader()?;

        let (batch_size, target_batch_size) = Self::calculate_batch_sizes(input, perf)?;

        let tally_capacity = perf.capacity(input.size());
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
                    &mut reader,
                    &mut buffer,
                    &mut reached_eof,
                    target_batch_size,
                    input,
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
    fn process_chunks(content: &str, boundaries: &[usize], case: Case) -> Self {
        boundaries
            .windows(2)
            .par_bridge()
            .filter_map(|window| {
                let (start, end) = (window[0], window[1]);
                (end > start).then(|| &content[start..end])
            })
            .fold(Self::new, |mut tally, chunk| {
                tally.add_words_from(chunk, case);
                tally
            })
            .reduce_with(Self::merge)
            .unwrap_or_default()
    }

    /// Reads the entire input into a string buffer.
    fn read_input_to_string(input: &Input, perf: &Performance) -> Result<String> {
        let buffer_capacity = perf.capacity(input.size());
        let mut buffer = String::with_capacity(buffer_capacity);
        input
            .reader()
            .with_context(|| format!("failed to create reader for input: {}", input.source()))?
            .read_to_string(&mut buffer)
            .with_context(|| format!("failed to read input into buffer: {}", input.source()))?;

        Ok(buffer)
    }

    /// Calculate batch sizes for streaming.
    fn calculate_batch_sizes(input: &Input, perf: &Performance) -> Result<(usize, usize)> {
        let stream_batch = Performance::stream_batch_size();
        let batch_size = usize::try_from(stream_batch)
            .map_err(|_| WordTallyError::BatchSizeExceeded { size: stream_batch })?;
        let input_size = input.size().unwrap_or_else(|| perf.base_stdin_size());
        let target_size = perf.stream_batch_size_for_input(input_size);
        let target_batch_size = usize::try_from(target_size)
            .map_err(|_| WordTallyError::BatchSizeExceeded { size: target_size })?;

        Ok((batch_size, target_batch_size))
    }

    /// Find chunk boundaries at whitespace with SIMD.
    fn chunk_boundaries(content: &str, num_chunks: usize) -> Vec<usize> {
        let content_len = content.len();
        let target_chunk_size = content_len.div_ceil(num_chunks);

        let mut boundaries = vec![0];
        let mut last_boundary = 0;

        for pos in memchr2_iter(b' ', b'\n', content.as_bytes()) {
            if pos - last_boundary >= target_chunk_size {
                let boundary = pos + 1;
                boundaries.push(boundary);
                last_boundary = boundary;
            }
        }

        // Add end boundary if needed
        if boundaries.last() != Some(&content_len) {
            boundaries.push(content_len);
        }

        boundaries
    }

    /// Find memory-mapped chunk boundaries using SIMD whitespace detection.
    fn mmap_boundaries(content: &str, num_chunks: usize) -> Vec<usize> {
        let content_len = content.len();
        let target_chunk_size = content_len.div_ceil(num_chunks);

        let mut boundaries = Vec::with_capacity(num_chunks + 1);
        boundaries.extend(
            iter::once(0).chain((1..=num_chunks).map(|i| i * target_chunk_size).map(
                |target_pos| {
                    if target_pos >= content_len {
                        // Last boundary is end of file if the proposed chunk is past the end
                        content_len
                    } else {
                        // Search backwards for whitespace (space or newline) with SIMD
                        let search_region = &content.as_bytes()[..target_pos];
                        memchr::memrchr2(b' ', b'\n', search_region)
                            .map_or(target_pos, |pos| pos + 1)
                    }
                },
            )),
        );
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

        let mut boundaries: Vec<usize> = iter::once(0)
            .chain(
                whitespace_positions
                    .iter()
                    .scan(0, |last_boundary, &pos| {
                        if pos - *last_boundary >= chunk_size {
                            *last_boundary = pos + 1;
                            Some(Some(pos + 1))
                        } else {
                            Some(None)
                        }
                    })
                    .flatten(),
            )
            .collect();

        // Add the end of buffer at end of file if needed
        if reached_eof && boundaries.last() < Some(&buffer.len()) {
            boundaries.push(buffer.len());
        }

        boundaries
    }

    /// Fill the buffer with data from the stream reader up to the target size.
    fn fill_stream_buffer(
        reader: &mut dyn BufRead,
        buffer: &mut Vec<u8>,
        reached_eof: &mut bool,
        target_size: usize,
        input: &Input,
    ) -> Result<()> {
        while buffer.len() < target_size && !*reached_eof {
            // Fill reader's internal buffer
            let available = reader
                .fill_buf()
                .with_context(|| format!("failed to read from: {}", input.source()))?;

            if available.is_empty() {
                *reached_eof = true;
                break;
            }

            // Copy only what's needed to reach target size
            let bytes_to_copy = available.len().min(target_size - buffer.len());
            buffer.extend_from_slice(&available[..bytes_to_copy]);
            // Mark bytes as consumed in reader
            reader.consume(bytes_to_copy);
        }

        Ok(())
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
                tally.add_words_from(remaining, case);
                Ok(Some((tally, buffer.len())))
            }
            _ => {
                // Multiple chunks: process in parallel
                let process_end = chunk_boundaries[chunk_boundaries.len() - 1];
                let content = Self::parse_utf8_slice(&buffer[..process_end])?;
                let batch_result = Self::process_chunks(content, &chunk_boundaries, case);
                Ok(Some((batch_result, process_end)))
            }
        }
    }

    /// Find the last valid UTF-8 character boundary at or before the given position.
    fn find_valid_boundary(buffer: &[u8], valid_len: usize) -> usize {
        if valid_len == 0 {
            return 0;
        }

        (0..=valid_len)
            .rev()
            .find(|&len| str::from_utf8(&buffer[..len]).is_ok())
            .unwrap_or(0)
    }

    /// Parse a UTF-8 slice with error recovery, truncating at last valid boundary on error.
    fn parse_utf8_slice(buffer: &[u8]) -> Result<&str> {
        str::from_utf8(buffer).or_else(|e| {
            let adjusted_len = Self::find_valid_boundary(buffer, e.valid_up_to());

            if adjusted_len == 0 {
                return Err(WordTallyError::Utf8 {
                    byte: e.valid_up_to(),
                    source: e,
                }
                .into());
            }

            str::from_utf8(&buffer[..adjusted_len]).map_err(|_| {
                WordTallyError::Utf8 {
                    byte: e.valid_up_to(),
                    source: e,
                }
                .into()
            })
        })
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
        Self {
            inner: HashMap::from_iter(iter),
        }
    }
}

impl Extend<(Word, Count)> for TallyMap {
    fn extend<T: IntoIterator<Item = (Word, Count)>>(&mut self, tally: T) {
        let iter = tally.into_iter();
        let (lower_bound, _) = iter.size_hint();

        self.inner.reserve(lower_bound);
        iter.for_each(|(word, count)| {
            self.inner
                .entry(word)
                .and_modify(|c| *c += count)
                .or_insert(count);
        });
    }
}
