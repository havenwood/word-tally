# Test organization

This directory contains all tests for the word-tally project.

## Structure

- **CLI Tests** (`cli_*.rs`): Tests for command-line interface functionality
  - `cli_basic_tests.rs`: Basic commands (version, help)
  - `cli_case_sort_tests.rs`: Case and sort options
  - `cli_env_tests.rs`: Environment variable tests
  - `cli_format_tests.rs`: Output format options (JSON, CSV, text)
  - `cli_output_tests.rs`: File output tests
  - `cli_pattern_tests.rs`: Pattern matching and filtering
  - `cli_verbose_tests.rs`: Verbose output tests
  - `cli_tests.rs`: Legacy CLI tests

- **Library Tests** (`tally_*.rs`): Tests for the core library functionality
  - `tally_core_tests.rs`: Core WordTally functionality
  - `tally_defaults_tests.rs`: Default implementations
  - `tally_filter_tests.rs`: Pattern and word filtering
  - `tally_io_mode_tests.rs`: I/O mode comparison tests
  - `tally_iterator_tests.rs`: Iterator implementation
  - `tally_serialization_tests.rs`: JSON/serde tests
  - `tally_serialization_error_tests.rs`: Serialization error handling

- **API Tests** (`api_tests.rs`): Public API integration tests
- **Error Tests** (`error_integration_tests.rs`): Error handling integration tests
- **Legacy Tests** (`lib_tests.rs`): Legacy library tests

## Module-specific tests

Tests for specific modules and components:
- `args_tests.rs`: Argument parsing tests
- `errors_tests.rs`: Error code mapping tests
- `filters_tests.rs`: Filter functionality tests
- `hash_tests.rs`: Hash implementation tests
- `input_tests.rs`: Input module tests
- `io_tests.rs`: I/O strategy tests
- `options_tests.rs`: Options configuration tests
- `output_tests.rs`: Output module tests
- `performance_tests.rs`: Performance settings tests
- `processing_tests.rs`: Processing mode tests
- `serialization_tests.rs`: Serialization module tests
- `threads_tests.rs`: Thread configuration tests
- `traits_tests.rs`: Trait implementation tests
- `verbose_tests.rs`: Verbose output tests

## Test utilities

Each test file is self-contained with local helper functions. This approach ensures simple compilation and avoids complex module dependencies. Common patterns include:
- `word_tally()` function for CLI command building
- `make_shared()` function for Arc wrapper creation
- Test-specific helper functions as needed

## Test data

The `fixtures/` directory contains test data files.

## Running tests

```bash
# Run all tests
cargo test

# Run specific test file
cargo test --test cli_basic_tests

# Run tests with verbose output
cargo test -- --nocapture

# Run tests in release mode
cargo test --release
```

## Writing tests

When adding new tests:

1. Place integration tests at the root of `tests/`
2. Name CLI tests with `cli_` prefix
3. Name library tests with `tally_` prefix
4. Use descriptive test function names
5. Document complex test scenarios with comments
6. Keep tests focused on a single aspect of functionality
