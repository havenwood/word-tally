//! A collection for tallying word counts using `HashMap`.

use std::{
    collections::{HashMap, hash_map},
    fmt::Display,
    io::{BufRead, Read},
    path::Path,
    str,
    sync::Arc,
};

use anyhow::{Context, Result};
use memchr::memchr_iter;
use memmap2::Mmap;
use rayon::prelude::*;
use unicode_segmentation::UnicodeSegmentation;

use crate::options::{
    Options, case::Case, io::Io, performance::Performance, processing::Processing,
};
use crate::{Count, Input, Word};

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

    /// Extends the tally map with word counts from a string slice.
    #[must_use]
    pub fn from_content(content: &str, case: Case, perf: &Performance) -> Self {
        let mut instance = Self::with_capacity(perf.chunk_capacity(content.len()));

        for word in content.unicode_words() {
            *instance.inner.entry(case.normalize(word)).or_insert(0) += 1;
        }

        instance
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
    pub fn extend_from_str(&mut self, content: &str, case: Case) {
        for word in content.unicode_words() {
            *self.inner.entry(case.normalize(word)).or_insert(0) += 1;
        }
    }

    /// Merge two tally maps, always merging the smaller into the larger.
    ///
    /// Returns the merged map containing the combined counts from both input maps.
    #[must_use]
    pub fn merge(mut self, other: Self) -> Self {
        // Always merge the smaller map into the larger for better performance
        if self.len() < other.len() {
            let mut merged = other;
            merged.extend(self);
            merged
        } else {
            self.extend(other);
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
                let Input::Mmap(mmap_arc, path) = input else {
                    unreachable!("This will only be called with `Input::Mmap(Arc<Mmap>, PathBuf)`.")
                };
                Self::par_mmap_count(mmap_arc, path, options)
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
        tally.extend_from_str(&content, options.case());

        Ok(tally)
    }

    /// Parallel buffered word tallying.
    ///
    /// Processes data in parallel with Rayon after loading entire content:
    /// - Uses memchr SIMD to find newline-aligned chunk boundaries
    /// - Trades higher memory usage for processing speed
    fn par_buffered_count(input: &Input, options: &Options) -> Result<Self> {
        let case = options.case();
        let perf = options.performance();
        let content = Self::read_input_to_string(input, perf)?;
        let content_len = content.len();
        let total_chunks = perf.total_chunks(content_len);

        // Find optimal chunk boundaries that align with newlines
        let boundaries = Self::simd_chunk_boundaries(&content, total_chunks);

        // Process content in parallel using the boundaries
        let tally = Self::process_content_in_chunks(&content, case, &boundaries);

        Ok(tally)
    }

    /// Sequential streamed word tallying.
    ///
    /// Streams one line at a time, avoiding needing to always hold the entire input in memory.
    fn streamed_count(input: &Input, options: &Options) -> Result<Self> {
        let perf = options.performance();
        let mut tally = Self::with_capacity(perf.capacity(input.size()));
        let reader = input.reader()?;

        reader
            .lines()
            .enumerate()
            .try_for_each(|(line_num, try_line)| {
                let line = try_line.with_context(|| {
                    format!(
                        "failed to read line {} from: {}",
                        line_num + 1,
                        input.source()
                    )
                })?;
                tally.extend_from_str(&line, options.case());

                Ok::<_, anyhow::Error>(())
            })?;

        Ok(tally)
    }

    /// Parallel, streamed word tallying.
    ///
    /// Processes data in parallel with Rayon while streaming:
    /// - Uses memchr SIMD to find newline-aligned chunk boundaries
    /// - Stream in chunks, without loading the entire input into memory
    /// - Balances performance and memory usage
    fn par_streamed_count(input: &Input, options: &Options) -> Result<Self> {
        let perf = options.performance();
        let case = options.case();
        let mut reader = input.reader()?;

        let buffer_size = crate::options::performance::Performance::STREAM_BUFFER_SIZE;
        let input_size = input.size().unwrap_or_else(|| perf.base_stdin_size_usize());
        let num_threads = rayon::current_num_threads();
        let total_chunks =
            num_threads * crate::options::performance::Performance::CHUNKS_PER_THREAD;
        let min_chunk_size = crate::options::performance::Performance::MIN_STREAM_CHUNK_SIZE;
        let target_chunk_size = input_size.div_ceil(total_chunks).max(min_chunk_size);

        let tally_capacity = perf.capacity(input.size());
        let mut tally = Self::with_capacity(tally_capacity);

        // Buffer to hold the current batch of data
        let mut buffer = Vec::with_capacity(buffer_size);
        let mut eof = false;

        while !eof {
            // Fill buffer until we have enough data or reach EOF
            if buffer.len() < target_chunk_size {
                // Read directly into a temporary buffer
                let mut read_buffer = vec![0; buffer_size.saturating_sub(buffer.len())];
                let bytes_read = reader
                    .read(&mut read_buffer)
                    .with_context(|| format!("failed to read from: {}", input.source()))?;

                if bytes_read == 0 {
                    eof = true;
                    if buffer.is_empty() {
                        break;
                    }
                } else {
                    // Add only the bytes we actually read
                    buffer.extend_from_slice(&read_buffer[..bytes_read]);

                    if !eof && buffer.len() < target_chunk_size {
                        continue;
                    }
                }
            }

            // Find all newline positions in a single pass
            let newline_positions: Vec<usize> = memchr_iter(b'\n', &buffer).collect();

            // If we have no newlines or not enough data, continue reading
            if newline_positions.is_empty() && !eof && buffer.len() < target_chunk_size {
                continue;
            }

            // Determine chunk boundaries based on newlines and target chunk size
            let mut chunk_boundaries = Vec::with_capacity(total_chunks + 1);
            chunk_boundaries.push(0);

            // Create boundaries at newlines near chunk size targets
            let mut last_boundary = 0;
            for pos in &newline_positions {
                if *pos - last_boundary >= target_chunk_size {
                    // Include the newline boundary in the chunk
                    chunk_boundaries.push(*pos + 1);
                    last_boundary = *pos + 1;
                }
            }

            // Add the end of buffer at end of file
            if eof
                && (chunk_boundaries.is_empty() || *chunk_boundaries.last().unwrap() < buffer.len())
            {
                chunk_boundaries.push(buffer.len());
            }

            if chunk_boundaries.len() > 1 {
                // Validate UTF-8 once for the entire processed section
                let process_end = *chunk_boundaries.last().unwrap();
                let content = Self::parse_utf8_slice(&buffer[..process_end], input)?;

                // Process chunks in parallel using boundaries similar to other parallel methods
                let batch_result = chunk_boundaries
                    .windows(2)
                    .par_bridge()
                    .fold(Self::new, |mut local_tally, window| {
                        let start = window[0];
                        let end = window[1];
                        if end > start {
                            local_tally.extend_from_str(&content[start..end], case);
                        }
                        local_tally
                    })
                    .reduce_with(Self::merge)
                    .unwrap_or_else(Self::default);

                // Merge the batch results into the main tally
                tally = tally.merge(batch_result);

                // Remove processed data from the buffer
                let last_boundary = *chunk_boundaries.last().unwrap();
                if last_boundary > 0 && last_boundary <= buffer.len() {
                    buffer.drain(..last_boundary);
                }
            } else if eof && !buffer.is_empty() {
                // Process any remaining data at end of file
                let remaining = Self::parse_utf8_slice(&buffer, input)?;
                tally.extend_from_str(remaining, case);
                buffer.clear();
            }
        }

        Ok(tally)
    }

    /// Parallel memory-mapped word tallying.
    ///
    /// Processes data in parallel with Rayon using memory-mapped slicing of files:
    /// - Uses memmap2 to access file contents without loading the entire file into memory
    /// - Optimized chunk boundary detection using direct slices with character boundary checks
    /// - Provides high performance with moderate memory usage
    ///
    /// # Limitations
    ///
    /// Memory mapping only works with seekable files (regular files on disk).
    /// It will not work with stdin, pipes, or other non-seekable sources.
    fn par_mmap_count(mmap: &Arc<Mmap>, path: &Path, options: &Options) -> Result<Self> {
        let perf = options.performance();
        let case = options.case();

        // Provide a view into the content rather than copying
        let content = Self::parse_utf8_slice(mmap, &path.display())?;
        let content_len = content.len();
        let total_chunks = perf.total_chunks(content_len);

        // Find optimal chunk boundaries that align with character boundaries
        let boundaries = Self::mmap_chunk_boundaries(content, total_chunks);

        // Process content in parallel using the boundaries
        let tally = Self::process_content_in_chunks(content, case, &boundaries);

        Ok(tally)
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

    /// Find chunk boundaries at newlines with SIMD.
    fn simd_chunk_boundaries(content: &str, total_chunks: usize) -> Vec<usize> {
        let content_len = content.len();
        let target_chunk_size = content_len.div_ceil(total_chunks);

        // Add an extra capacity for the closing boundary
        let mut boundaries = Vec::with_capacity(total_chunks + 1);
        boundaries.push(0);

        if !content.is_empty() {
            let mut last_boundary = 0;

            for pos in memchr_iter(b'\n', content.as_bytes()) {
                if pos - last_boundary >= target_chunk_size && content.is_char_boundary(pos + 1) {
                    boundaries.push(pos + 1);
                    last_boundary = pos + 1;
                }
            }
        }

        if *boundaries.last().unwrap_or(&0) < content_len {
            boundaries.push(content_len);
        }

        boundaries
    }

    /// Find memory-mapped chunk boundaries with direct character boundary checks.
    fn mmap_chunk_boundaries(content: &str, num_chunks: usize) -> Vec<usize> {
        let total_size = content.len();
        let chunk_size = total_size.div_ceil(num_chunks);

        if total_size == 0 {
            return vec![0];
        }

        let mut boundaries = Vec::with_capacity(num_chunks + 1);
        boundaries.push(0);

        boundaries.extend((1..num_chunks).map(|i| {
            // Find next valid UTF-8 character boundary after a chunk of bytes
            let start_pos = i * chunk_size;
            if start_pos >= total_size {
                total_size
            } else {
                (start_pos..total_size)
                    .find(|&pos| content.is_char_boundary(pos))
                    .unwrap_or(total_size)
            }
        }));

        if *boundaries.last().unwrap_or(&0) < total_size {
            boundaries.push(total_size);
        }

        boundaries
    }

    /// Process content in parallel chunks using pre-calculated boundaries.
    ///
    /// Takes content and chunk boundaries, processes each chunk in parallel,
    /// then merges results. Used by both memory-mapped and buffered parallel modes.
    fn process_content_in_chunks(content: &str, case: Case, boundaries: &[usize]) -> Self {
        boundaries
            .windows(2)
            .par_bridge()
            .fold(Self::new, |mut local_tally, window| {
                local_tally.extend_from_str(&content[window[0]..window[1]], case);
                local_tally
            })
            .reduce_with(Self::merge)
            .unwrap_or_else(Self::default)
    }

    /// Parse a UTF-8 slice with error recovery, truncating at last valid boundary on error.
    fn parse_utf8_slice<'a, D: Display>(buffer: &'a [u8], input_name: &D) -> Result<&'a str> {
        str::from_utf8(buffer).or_else(|e| {
            let valid_len = e.valid_up_to();
            let adjusted_len = Self::find_valid_boundary(buffer, valid_len);

            match adjusted_len {
                0 => Err(e).with_context(|| {
                    format!(
                        "invalid UTF-8 at byte offset {} from: {}",
                        e.valid_up_to(),
                        input_name
                    )
                }),
                len => str::from_utf8(&buffer[..len])
                    .map_err(|_| e)
                    .with_context(|| {
                        format!(
                            "invalid UTF-8 at byte offset {} from: {}",
                            e.valid_up_to(),
                            input_name
                        )
                    }),
            }
        })
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

        for (word, count) in iter {
            *self.inner.entry(word).or_insert(0) += count;
        }
    }
}
