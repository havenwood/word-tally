# Test Organization

All tests for word-tally are located in this directory, organized by functionality.

## Test Files

### CLI Tests (`cli_*.rs`)
Command-line interface tests using `assert_cmd`:
- `cli_delimiter_output_tests.rs` - Delimiter output formatting
- `cli_env_tests.rs` - Environment variable handling
- `cli_pattern_tests.rs` - Pattern matching (include/exclude)
- `cli_serialization_tests.rs` - Output formats (JSON, CSV, text)
- `cli_tests.rs` - General CLI functionality
- `cli_verbose_tests.rs` - Verbose output formatting

### Core Library Tests (`tally_*.rs`)
Tests for the WordTally struct and core functionality:
- `tally_core_tests.rs` - Core operations (sorting, filtering)
- `tally_filter_tests.rs` - Word filtering logic
- `tally_iterator_tests.rs` - Iterator implementations
- `tally_map_tests.rs` - TallyMap operations
- `tally_serialization_tests.rs` - Serialization/deserialization

### Module Tests (`{module}_tests.rs`)
Tests for specific modules:
- `args_tests.rs` - CLI argument parsing
- `case_tests.rs` - Case normalization functionality
- `delimiters_tests.rs` - Delimiter and Delimiters types
- `exit_code_tests.rs` - Exit code mappings
- `filters_tests.rs` - Filter configurations
- `hash_tests.rs` - Hash implementations
- `buffered_tests.rs` - Buffered input functionality and file I/O
- `mapped_tests.rs` - Mapped input functionality and memory mapping
- `io_tests.rs` - I/O strategies
- `options_tests.rs` - Options configuration
- `output_tests.rs` - Output destinations
- `patterns_tests.rs` - Regex patterns
- `performance_tests.rs` - Performance settings
- `serialization_tests.rs` - Serialization formats
- `threads_tests.rs` - Thread configuration
- `verbose_tests.rs` - Verbose output logic

### Integration Tests
- `api_tests.rs` - Public API usage examples
- `error_integration_tests.rs` - Error handling scenarios
- `lib_tests.rs` - Comprehensive library tests
- `multi_file_io_tests.rs` - Multiple file handling
- `traits_tests.rs` - Trait implementations

## Running Tests

```bash
# All tests
cargo test

# Specific test file
cargo test --test delimiters_tests

# With output
cargo test -- --nocapture

# In release mode
cargo test --release
```

## Test Guidelines

- Each test file is self-contained
- Helper functions are defined locally
- No shared test utilities module
- Tests are named descriptively
- Doctests are minimal