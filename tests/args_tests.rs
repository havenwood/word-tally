use assert_cmd::Command;
use predicates::str::contains;
use std::fs;
use tempfile::NamedTempFile;

//
// Default Values Testing
//

#[test]
fn test_args_default_input() {
    // Default input should be stdin ("-")
    Command::cargo_bin("word-tally")
        .unwrap()
        .arg("-v")
        .write_stdin("test")
        .assert()
        .success()
        .stderr(contains("source -"));
}

#[test]
fn test_args_default_io() {
    // Default IO should be streamed
    Command::cargo_bin("word-tally")
        .unwrap()
        .arg("-v")
        .write_stdin("test")
        .assert()
        .success()
        .stderr(contains("io streamed"));
}

#[test]
fn test_args_default_case() {
    // Default case should be lower
    Command::cargo_bin("word-tally")
        .unwrap()
        .arg("-v")
        .write_stdin("test")
        .assert()
        .success()
        .stderr(contains("case lower"));
}

#[test]
fn test_args_default_sort() {
    // Default sort should be desc
    Command::cargo_bin("word-tally")
        .unwrap()
        .arg("-v")
        .write_stdin("test")
        .assert()
        .success()
        .stderr(contains("order desc"));
}

#[test]
fn test_args_default_format() {
    // Default format should be text (no special format in verbose output)
    Command::cargo_bin("word-tally")
        .unwrap()
        .arg("-v")
        .write_stdin("test")
        .assert()
        .success()
        .stderr(contains("delimiter \" \""));
}

#[test]
fn test_args_default_delimiter() {
    // Default delimiter should be a space
    Command::cargo_bin("word-tally")
        .unwrap()
        .arg("-v")
        .write_stdin("test")
        .assert()
        .success()
        .stderr(contains("delimiter \" \""));
}

#[test]
fn test_args_default_processing() {
    // Default processing should be sequential
    Command::cargo_bin("word-tally")
        .unwrap()
        .arg("-v")
        .write_stdin("test")
        .assert()
        .success()
        .stderr(contains("processing sequential"));
}

#[test]
fn test_args_default_filters() {
    // Default filters should all be none
    Command::cargo_bin("word-tally")
        .unwrap()
        .arg("-v")
        .write_stdin("test")
        .assert()
        .success()
        .stderr(contains("min-chars none"))
        .stderr(contains("min-count none"))
        .stderr(contains("exclude-words none"))
        .stderr(contains("exclude-patterns none"))
        .stderr(contains("include-patterns none"));
}

#[test]
fn test_args_default_verbose() {
    // Default verbose should be false (no verbose output without -v)
    Command::cargo_bin("word-tally")
        .unwrap()
        .write_stdin("test")
        .assert()
        .success()
        .stdout(contains("test 1"))
        .stderr(predicates::str::is_empty());
}

#[test]
fn test_args_default_output() {
    // Default output should be stdout (no file output)
    Command::cargo_bin("word-tally")
        .unwrap()
        .write_stdin("test")
        .assert()
        .success()
        .stdout(contains("test 1"));
}

#[test]
fn test_args_all_defaults() {
    // Comprehensive test with all defaults
    Command::cargo_bin("word-tally")
        .unwrap()
        .write_stdin("This is a test")
        .assert()
        .success()
        .stdout(contains("this 1"))
        .stdout(contains("is 1"))
        .stdout(contains("a 1"))
        .stdout(contains("test 1"));
}

#[test]
fn test_args_defaults_with_minimal_flags() {
    // Test that defaults work with minimal flags
    Command::cargo_bin("word-tally")
        .unwrap()
        .arg("-v")
        .write_stdin("Test")
        .assert()
        .success()
        .stdout(contains("test 1"))
        .stderr(contains("source -"))
        .stderr(contains("total-words 1"))
        .stderr(contains("unique-words 1"))
        .stderr(contains("delimiter \" \""))
        .stderr(contains("case lower"))
        .stderr(contains("order desc"))
        .stderr(contains("processing sequential"))
        .stderr(contains("io streamed"))
        .stderr(contains("min-chars none"))
        .stderr(contains("min-count none"))
        .stderr(contains("exclude-words none"))
        .stderr(contains("exclude-patterns none"))
        .stderr(contains("include-patterns none"));
}

//
// Basic Argument Parsing
//

#[test]
fn test_get_options() {
    let assert = Command::cargo_bin("word-tally")
        .unwrap()
        .arg("--case=upper")
        .arg("--sort=asc")
        .arg("--format=json")
        .arg("--min-chars=3")
        .arg("--parallel")
        .arg("-v")
        .write_stdin("hello world")
        .assert();

    assert
        .success()
        .stderr(contains("\"case\":\"upper\""))
        .stderr(contains("\"order\":\"asc\""))
        .stderr(contains("\"minChars\":\"3\""));
}

#[test]
fn test_get_performance() {
    let assert = Command::cargo_bin("word-tally")
        .unwrap()
        .arg("--parallel")
        .arg("-v")
        .write_stdin("hello world")
        .assert();

    assert.success();
}

#[test]
fn test_get_formatting() {
    let assert = Command::cargo_bin("word-tally")
        .unwrap()
        .arg("--case=upper")
        .arg("--sort=asc")
        .arg("--format=json")
        .arg("-v")
        .write_stdin("hello")
        .assert();

    assert
        .success()
        .stderr(contains("\"case\":\"upper\""))
        .stderr(contains("\"order\":\"asc\""));
}

#[test]
fn test_get_filters() {
    let assert = Command::cargo_bin("word-tally")
        .unwrap()
        .arg("--min-chars=3")
        .arg("--min-count=2")
        .arg("--exclude-words=hello,world")
        .arg("-v")
        .write_stdin("hello world")
        .assert();

    assert
        .success()
        .stderr(contains("min-chars 3"))
        .stderr(contains("min-count 2"))
        .stderr(contains("exclude-words hello,world"));
}

#[test]
fn test_get_filters_with_patterns() {
    let assert = Command::cargo_bin("word-tally")
        .unwrap()
        .arg("--include=^h")
        .arg("--exclude=\\d+")
        .arg("-v")
        .write_stdin("hello 123 world")
        .assert();

    assert.success().stderr(contains("exclude-patterns"));
}

//
// Shorthand Flags Testing
//

#[test]
fn test_io_shorthand_flag() {
    let assert = Command::cargo_bin("word-tally")
        .unwrap()
        .arg("-I=buffered")
        .arg("-v")
        .write_stdin("hello world")
        .assert();

    assert.success().stderr(contains("io buffered"));
}

#[test]
fn test_args_shorthand_flags() {
    let assert = Command::cargo_bin("word-tally")
        .unwrap()
        .arg("-c=upper")
        .arg("-s=asc")
        .arg("-f=json")
        .arg("-d=,")
        .arg("-m=3")
        .arg("-M=2")
        .arg("-p")
        .arg("-v")
        .write_stdin("hello world test")
        .assert();

    assert
        .success()
        .stderr(contains("\"case\":\"upper\""))
        .stderr(contains("\"order\":\"asc\""))
        .stderr(contains("\"minChars\":\"3\""))
        .stderr(contains("\"minCount\":\"2\""));
}

#[test]
fn test_args_exclude_words_list() {
    Command::cargo_bin("word-tally")
        .unwrap()
        .arg("-E=the,a,an,and,or")
        .arg("-v")
        .write_stdin("the quick brown fox and a lazy dog")
        .assert()
        .success()
        .stderr(contains("exclude-words the,a,an,and,or"));
}

#[test]
fn test_args_multiple_include_patterns() {
    Command::cargo_bin("word-tally")
        .unwrap()
        .arg("-i=^h")
        .arg("-i=^w")
        .arg("-v")
        .write_stdin("hello world test")
        .assert()
        .success()
        .stderr(contains("include-patterns ^h,^w"));
}

#[test]
fn test_args_multiple_exclude_patterns() {
    Command::cargo_bin("word-tally")
        .unwrap()
        .arg("-x=ing$")
        .arg("-x=ed$")
        .arg("-v")
        .write_stdin("testing tested test")
        .assert()
        .success()
        .stderr(contains("exclude-patterns ing$,ed$"));
}

#[test]
fn test_args_output_file_shorthand() {
    let temp_file = tempfile::NamedTempFile::new().unwrap();

    Command::cargo_bin("word-tally")
        .unwrap()
        .arg("-o")
        .arg(temp_file.path())
        .write_stdin("test word")
        .assert()
        .success();

    let content = std::fs::read_to_string(temp_file.path()).unwrap();
    assert!(content.contains("test"));
    assert!(content.contains("word"));
}

//
// Input/Output Testing
//

#[test]
fn test_args_input_from_file() {
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(&temp_file, "test content").unwrap();

    let filename = temp_file.path().file_name().unwrap().to_str().unwrap();

    Command::cargo_bin("word-tally")
        .unwrap()
        .arg(temp_file.path())
        .arg("-v")
        .assert()
        .success()
        .stderr(contains(format!("source {}", filename)));
}

#[test]
fn test_args_output_to_file() {
    let temp_file = NamedTempFile::new().unwrap();

    Command::cargo_bin("word-tally")
        .unwrap()
        .arg("--output")
        .arg(temp_file.path())
        .write_stdin("test word test")
        .assert()
        .success();

    let content = fs::read_to_string(temp_file.path()).unwrap();
    assert!(content.contains("test 2"));
    assert!(content.contains("word 1"));
}

#[test]
fn test_args_stdin_explicit() {
    Command::cargo_bin("word-tally")
        .unwrap()
        .arg("-")
        .arg("-v")
        .write_stdin("test")
        .assert()
        .success()
        .stderr(contains("source -"));
}

//
// Verbose Testing
//

#[test]
fn test_args_verbose_flag() {
    Command::cargo_bin("word-tally")
        .unwrap()
        .arg("-v")
        .write_stdin("test")
        .assert()
        .success()
        .stderr(contains("source"));
}

//
// Comprehensive Filter Testing
//

#[test]
fn test_args_filters_comprehensive() {
    Command::cargo_bin("word-tally")
        .unwrap()
        .arg("--min-chars=5")
        .arg("--min-count=1")
        .arg("--exclude-words=hello,world")
        .arg("--include=^t")
        .arg("--exclude=ing$")
        .arg("-v")
        .write_stdin("hello testing test world tiny")
        .assert()
        .success()
        .stderr(contains("min-chars 5"))
        .stderr(contains("exclude-words hello,world"))
        .stderr(contains("include-patterns ^t"))
        .stderr(contains("exclude-patterns ing$"));
}

#[test]
fn test_args_escaped_words() {
    Command::cargo_bin("word-tally")
        .unwrap()
        .arg("--exclude-words=\\t,\\n")
        .arg("-v")
        .write_stdin("test\ttab\nnewline")
        .assert()
        .success()
        .stderr(contains("exclude-words"));
}

#[test]
fn test_args_multiple_patterns() {
    Command::cargo_bin("word-tally")
        .unwrap()
        .arg("--include=^t")
        .arg("--include=^w")
        .arg("--exclude=st$")
        .arg("--exclude=rd$")
        .arg("-v")
        .write_stdin("test word testing wording")
        .assert()
        .success()
        .stderr(contains("include-patterns ^t,^w"))
        .stderr(contains("exclude-patterns st$,rd$"));
}

//
// Comprehensive Options Testing
//

#[test]
fn test_args_options_comprehensive() {
    Command::cargo_bin("word-tally")
        .unwrap()
        .arg("--case=lower")
        .arg("--sort=desc")
        .arg("--io=buffered")
        .arg("--format=csv")
        .arg("--delimiter=;")
        .arg("--parallel")
        .arg("-v")
        .write_stdin("Test TEST test")
        .assert()
        .success()
        .stderr(contains("metric,value"))
        .stderr(contains("processing,parallel"));
}

//
// All Enum Mode Testing
//

#[test]
fn test_args_all_io_modes() {
    let test_inputs = [
        ("streamed", "streamed"),
        ("buffered", "buffered"),
        ("mmap", "memory-mapped"),
    ];

    for (input, expected) in &test_inputs {
        Command::cargo_bin("word-tally")
            .unwrap()
            .arg(format!("--io={}", input))
            .arg("-v")
            .write_stdin("test")
            .assert()
            .success()
            .stderr(contains(format!("io {}", expected)));
    }
}

#[test]
fn test_args_case_modes() {
    let test_cases = ["original", "upper", "lower"];

    for case in &test_cases {
        Command::cargo_bin("word-tally")
            .unwrap()
            .arg(format!("--case={}", case))
            .arg("-v")
            .write_stdin("Test")
            .assert()
            .success()
            .stderr(contains(format!("case {}", case)));
    }
}

#[test]
fn test_args_sort_modes() {
    let test_sorts = ["unsorted", "asc", "desc"];

    for sort in &test_sorts {
        Command::cargo_bin("word-tally")
            .unwrap()
            .arg(format!("--sort={}", sort))
            .arg("-v")
            .write_stdin("test")
            .assert()
            .success()
            .stderr(contains(format!("order {}", sort)));
    }
}

#[test]
fn test_args_format_modes() {
    let test_formats = ["text", "json", "csv"];

    for format in &test_formats {
        Command::cargo_bin("word-tally")
            .unwrap()
            .arg(format!("--format={}", format))
            .arg("-v")
            .write_stdin("test")
            .assert()
            .success()
            .stderr(contains("source"));
    }
}

//
// Edge Cases
//

#[test]
fn test_args_delimiter_edge_cases() {
    // Empty delimiter
    Command::cargo_bin("word-tally")
        .unwrap()
        .arg("--delimiter=")
        .arg("--format=text")
        .write_stdin("test word")
        .assert()
        .success()
        .stdout(contains("test1"))
        .stdout(contains("word1"));

    // Multi-char delimiter
    Command::cargo_bin("word-tally")
        .unwrap()
        .arg("--delimiter= => ")
        .arg("--format=text")
        .write_stdin("test word")
        .assert()
        .success()
        .stdout(contains("test => 1"))
        .stdout(contains("word => 1"));
}

//
// Error Handling
//

#[test]
fn test_args_io_bytes_error() {
    Command::cargo_bin("word-tally")
        .unwrap()
        .arg("--io=bytes")
        .write_stdin("test")
        .assert()
        .failure()
        .stderr(contains("invalid value 'bytes'"));
}

#[test]
fn test_args_invalid_min_count() {
    Command::cargo_bin("word-tally")
        .unwrap()
        .arg("--min-count=invalid")
        .write_stdin("test")
        .assert()
        .failure()
        .stderr(contains("invalid value"));
}

#[test]
fn test_args_invalid_enum_values() {
    // Invalid case
    Command::cargo_bin("word-tally")
        .unwrap()
        .arg("--case=invalid")
        .write_stdin("test")
        .assert()
        .failure()
        .stderr(contains("invalid value"));

    // Invalid sort
    Command::cargo_bin("word-tally")
        .unwrap()
        .arg("--sort=invalid")
        .write_stdin("test")
        .assert()
        .failure()
        .stderr(contains("invalid value"));
}

#[test]
fn test_args_invalid_regex_exclude() {
    Command::cargo_bin("word-tally")
        .unwrap()
        .arg("--exclude=[")
        .write_stdin("test")
        .assert()
        .failure()
        .stderr(contains("failed to create exclude patterns"));
}

#[test]
fn test_args_invalid_regex_include() {
    Command::cargo_bin("word-tally")
        .unwrap()
        .arg("--include=(?P<")
        .write_stdin("test")
        .assert()
        .failure()
        .stderr(contains("failed to create include patterns"));
}

//
// Environment Interaction
//

#[test]
fn test_args_environment_interaction() {
    // Test that args work with environment variables
    Command::cargo_bin("word-tally")
        .unwrap()
        .env("WORD_TALLY_TALLY_CAPACITY", "100")
        .arg("-v")
        .write_stdin("test")
        .assert()
        .success()
        .stderr(contains("source"));
}
