//! A collection for tallying word counts using `HashMap`.

use std::{
    collections::{HashMap, hash_map},
    io::{BufRead, Read},
    mem,
    path::Path,
    str,
    sync::Arc,
};

use anyhow::{Context, Result};
use memmap2::Mmap;
use rayon::prelude::*;
use unicode_segmentation::UnicodeSegmentation;

use crate::options::case::Case;
use crate::options::io::Io;
use crate::options::processing::Processing;
use crate::options::{Options, performance::Performance};
use crate::{Count, Input, Word};

/// Map for tracking word counts with non-deterministic iteration order.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TallyMap {
    inner: HashMap<Word, Count>,
}

impl TallyMap {
    /// Creates a new empty `TallyMap`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new `TallyMap` with the specified capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: HashMap::with_capacity(capacity),
        }
    }

    /// Returns the number of unique words in the map.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns true if the map contains no words.
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
    pub fn extend_from_str(&mut self, content: &str, case: Case) {
        for word in content.unicode_words() {
            let normalized = case.normalize(word);
            *self.inner.entry(normalized).or_insert(0) += 1;
        }
    }

    /// Merge two tally maps, always merging the smaller into the larger.
    ///
    /// Returns the merged map containing the combined counts from both input maps.
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

    //
    // Sequential I/O
    //

    /// Sequential streamed word tallying.
    ///
    /// Streams one line at a time, avoiding needing to always hold the entire input in memory.
    fn streamed_count(input: &Input, options: &Options) -> Result<Self> {
        let reader = input.reader()?;
        let mut tally = Self::with_capacity(options.performance().capacity(input.size()));

        reader
            .lines()
            .enumerate()
            .try_for_each(|(line_num, try_line)| {
                let line = try_line.with_context(|| {
                    format!("failed to read line {} from: {}", line_num + 1, input)
                })?;
                tally.extend_from_str(&line, options.case());

                Ok::<_, anyhow::Error>(())
            })?;

        Ok(tally)
    }

    /// Sequential buffered word tallying.
    ///
    /// Reads the entire input into memory before processing sequentially.
    fn buffered_count(input: &Input, options: &Options) -> Result<Self> {
        let perf = options.performance();
        let case = options.case();
        let content = Self::read_input_to_string(input, perf)?;
        let capacity = perf.capacity(Some(content.len()));
        let mut tally = Self::with_capacity(capacity);

        tally.extend_from_str(&content, case);

        Ok(tally)
    }

    //
    // Parallel I/O
    //

    /// Parallel, streamed word tallying.
    ///
    /// Streams batches of lines, processing the batches in parallel. Avoids always needing to
    /// hold the entire input in memory. Uses memory-based batching for more consistent workloads.
    fn par_streamed_count(input: &Input, options: &Options) -> Result<Self> {
        let perf = options.performance();
        let case = options.case();
        let reader = input.reader()?;
        let estimated_lines_per_chunk = perf.lines_per_chunk();
        let mut tally = Self::with_capacity(perf.capacity(input.size()));

        // Helper closure to process accumulated lines and merge into tally
        let mut process_batch = |batch: Vec<String>| {
            if batch.is_empty() {
                return;
            }

            // Calculate batch size in bytes for better capacity estimation
            let batch_size_bytes = batch.iter().map(|s| s.len()).sum();
            let batch_capacity = perf.capacity(Some(batch_size_bytes));

            // Process batch in parallel - let Rayon handle work distribution
            let batch_result = batch
                .chunks(rayon::current_num_threads() * 2)
                .par_bridge()
                .map(|chunk| {
                    let mut local_tally = Self::with_capacity(batch_capacity);
                    for line in chunk {
                        local_tally.extend_from_str(line, case);
                    }
                    local_tally
                })
                .reduce(|| Self::with_capacity(batch_capacity), Self::merge);

            // Extend tally with batch results (reserve is handled in extend)
            tally.extend(batch_result);
        };

        let mut batch_of_lines = Vec::with_capacity(estimated_lines_per_chunk);
        let mut accumulated_bytes = 0;

        reader
            .lines()
            .enumerate()
            .try_for_each(|(line_num, try_line)| {
                let line = try_line.with_context(|| {
                    format!("failed to read line {} from: {}", line_num + 1, input)
                })?;

                // Track memory used by this line
                accumulated_bytes += line.len();
                batch_of_lines.push(line);

                // Process batch when it reaches target memory threshold
                if accumulated_bytes >= perf.chunk_size() {
                    // Last batch's size rather than estimation to better match input pattern
                    let current_batch_size = batch_of_lines.len();
                    // Swap out the full batch for an empty one, reusing the previous capacity
                    let full_batch_of_lines =
                        mem::replace(&mut batch_of_lines, Vec::with_capacity(current_batch_size));
                    process_batch(full_batch_of_lines);
                    accumulated_bytes = 0;
                }

                Ok::<_, anyhow::Error>(())
            })?;

        // Process any remaining lines in the final batch
        if !batch_of_lines.is_empty() {
            process_batch(batch_of_lines);
        }

        Ok(tally)
    }

    /// Parallel memory-mapped word tallying.
    ///
    /// Uses a simple divide-and-conquer approach for parallel processing with mmap:
    /// - Divids content into chunks at UTF-8 character boundaries
    /// - Direct memory-mapped file content access with OS-managed paging
    /// - Process full chunks without line-by-line iteration
    ///
    /// An alternative single-pass approach could begin processing chunks during.
    /// boundary detection and not store chunk indices, but would need more complex.
    /// thread coordination.
    fn par_mmap_count(mmap: &Arc<Mmap>, path: &Path, options: &Options) -> Result<Self> {
        let perf = options.performance();
        let case = options.case();
        let chunk_size = perf.chunk_size();

        // This provides a view into the content rather than copying
        let content = match str::from_utf8(mmap) {
            Ok(content) => content,
            Err(e) => {
                return Err(e).with_context(|| {
                    format!(
                        "invalid UTF-8 at byte offset {} from: {}",
                        e.valid_up_to(),
                        path.display()
                    )
                });
            }
        };
        let total_size = content.len();
        let num_chunks = total_size.div_ceil(chunk_size);

        // Create chunk boundaries with first (0) and last (total_size) elements, plus interior boundaries
        let chunk_boundaries = {
            let mut boundaries = Vec::with_capacity(num_chunks + 1);
            // First boundary
            boundaries.push(0);

            boundaries.extend((1..num_chunks).map(|i| {
                // Find next valid UTF-8 character boundary after `chunk_size` bytes using an iterator
                (i * chunk_size..)
                    .find(|&pos| pos >= total_size || content.is_char_boundary(pos))
                    .unwrap_or(total_size)
            }));

            // Last boundary
            boundaries.push(total_size);

            boundaries
        };

        // Calculate optimal capacities for better memory usage
        let avg_chunk_size = total_size / num_chunks.max(1);
        let per_chunk_capacity = perf.capacity(Some(avg_chunk_size));

        // Calculate optimal capacity for the final reduced `TallyMap`
        let reduce_capacity = perf.capacity(Some(mmap.len()));

        let tally = chunk_boundaries
            .windows(2)
            .par_bridge()
            .map(|window| {
                let chunk = &content[window[0]..window[1]];
                let mut local_tally = Self::with_capacity(per_chunk_capacity);

                // Extract words from chunks, without heed for newlines
                local_tally.extend_from_str(chunk, case);

                local_tally
            })
            .reduce(|| Self::with_capacity(reduce_capacity), Self::merge);

        Ok(tally)
    }

    /// Parallel buffered word tallying.
    ///
    /// Reads the entire input into memory before processing in parallel.
    fn par_buffered_count(input: &Input, options: &Options) -> Result<Self> {
        let case = options.case();
        let perf = options.performance();
        let content = Self::read_input_to_string(input, perf)?;

        let tally = content
            .par_lines()
            .fold(
                || Self::with_capacity(perf.capacity_per_thread()),
                |mut tally, line| {
                    tally.extend_from_str(line, case);
                    tally
                },
            )
            .reduce_with(Self::merge)
            .unwrap_or_else(|| Self::with_capacity(perf.capacity(Some(content.len()))));

        Ok(tally)
    }

    /// Reads the entire input into a string buffer.
    fn read_input_to_string(input: &Input, perf: &Performance) -> Result<String> {
        // Use actual size if available; otherwise use base_stdin_size() for unknown inputs
        let buffer_capacity = input.size().unwrap_or_else(|| perf.base_stdin_size());

        // Read entire input into memory
        let mut buffer = String::with_capacity(buffer_capacity);
        input
            .reader()
            .with_context(|| format!("failed to create reader for input: {}", input))?
            .read_to_string(&mut buffer)
            .with_context(|| format!("failed to read input into buffer: {}", input))?;

        Ok(buffer)
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
