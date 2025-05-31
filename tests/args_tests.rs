//! Tests for command-line argument parsing.

use assert_cmd::Command;
use predicates::str::contains;
use std::fs;
use tempfile::NamedTempFile;

fn word_tally() -> Command {
    Command::cargo_bin("word-tally").expect("run test")
}

fn test_verbose_command(args: &[&str], stdin: &str, expected_patterns: &[&str]) {
    let mut cmd = word_tally();
    for arg in args {
        cmd.arg(*arg);
    }
    let mut assert = cmd.write_stdin(stdin).assert().success();

    for pattern in expected_patterns {
        assert = assert.stderr(contains(*pattern));
    }
}

fn test_basic_command(args: &[&str], stdin: &str, expected_stdout: &[&str]) {
    let mut cmd = word_tally();
    for arg in args {
        cmd.arg(*arg);
    }
    let mut assert = cmd.write_stdin(stdin).assert().success();

    for pattern in expected_stdout {
        assert = assert.stdout(contains(*pattern));
    }
}

//
// Default Values Testing
//

#[test]
fn test_args_default_input() {
    test_verbose_command(&["-v"], "test", &["source -"]);
}

#[test]
fn test_args_default_io() {
    test_verbose_command(&["-v"], "test", &["io parallel-stream"]);
}

#[test]
fn test_args_default_case() {
    test_verbose_command(&["-v"], "test", &["case original"]);
}

#[test]
fn test_args_default_sort() {
    test_verbose_command(&["-v"], "test", &["order desc"]);
}

#[test]
fn test_args_default_format() {
    test_verbose_command(&["-v"], "test", &["delimiter \" \""]);
}

#[test]
fn test_args_default_delimiter() {
    test_verbose_command(&["-v"], "test", &["delimiter \" \""]);
}

#[test]
fn test_args_default_processing() {
    test_verbose_command(&["-v"], "test", &["io parallel-stream"]);
}

#[test]
fn test_args_default_filters() {
    // Default filters should all be none
    Command::cargo_bin("word-tally")
        .expect("arguments should be valid")
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
        .expect("arguments should be valid")
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
        .expect("arguments should be valid")
        .write_stdin("test")
        .assert()
        .success()
        .stdout(contains("test 1"));
}

#[test]
fn test_args_all_defaults() {
    test_basic_command(&[], "This is a test", &["This 1", "is 1", "a 1", "test 1"]);
}

#[test]
fn test_args_defaults_with_minimal_flags() {
    // Test that defaults work with minimal flags
    Command::cargo_bin("word-tally")
        .expect("arguments should be valid")
        .arg("-v")
        .write_stdin("Test")
        .assert()
        .success()
        .stdout(contains("Test 1"))
        .stderr(contains("source -"))
        .stderr(contains("total-words 1"))
        .stderr(contains("unique-words 1"))
        .stderr(contains("delimiter \" \""))
        .stderr(contains("case original"))
        .stderr(contains("order desc"))
        .stderr(contains("io parallel-stream"))
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
        .expect("arguments should be valid")
        .arg("--case=upper")
        .arg("--sort=asc")
        .arg("--format=json")
        .arg("--min-chars=3")
        .arg("-v")
        .write_stdin("hello world")
        .assert();

    assert
        .success()
        .stderr(contains("\"case\":\"upper\""))
        .stderr(contains("\"order\":\"asc\""))
        .stderr(contains("\"minChars\":3"));
}

#[test]
fn test_get_performance() {
    let assert = Command::cargo_bin("word-tally")
        .expect("arguments should be valid")
        .arg("-v")
        .write_stdin("hello world")
        .assert();

    assert.success();
}

#[test]
fn test_get_formatting() {
    let assert = Command::cargo_bin("word-tally")
        .expect("arguments should be valid")
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
        .expect("arguments should be valid")
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
        .expect("arguments should be valid")
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
        .expect("arguments should be valid")
        .arg("-I=parallel-in-memory")
        .arg("-v")
        .write_stdin("hello world")
        .assert();

    assert.success().stderr(contains("io parallel-in-memory"));
}

#[test]
fn test_args_shorthand_flags() {
    let assert = Command::cargo_bin("word-tally")
        .expect("arguments should be valid")
        .arg("-c=upper")
        .arg("-s=asc")
        .arg("-f=json")
        .arg("-d=,")
        .arg("-m=3")
        .arg("-n=2")
        .arg("-v")
        .write_stdin("hello world test")
        .assert();

    assert
        .success()
        .stderr(contains("\"case\":\"upper\""))
        .stderr(contains("\"order\":\"asc\""))
        .stderr(contains("\"minChars\":3"))
        .stderr(contains("\"minCount\":2"));
}

#[test]
fn test_args_exclude_words_list() {
    Command::cargo_bin("word-tally")
        .expect("arguments should be valid")
        .arg("-w=the,a,an,and,or")
        .arg("-v")
        .write_stdin("the quick brown fox and a lazy dog")
        .assert()
        .success()
        .stderr(contains("exclude-words the,a,an,and,or"));
}

#[test]
fn test_args_multiple_include_patterns() {
    Command::cargo_bin("word-tally")
        .expect("arguments should be valid")
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
        .expect("arguments should be valid")
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
    let temp_file = tempfile::NamedTempFile::new().expect("create temp file");

    Command::cargo_bin("word-tally")
        .expect("arguments should be valid")
        .arg("-o")
        .arg(temp_file.path())
        .write_stdin("test word")
        .assert()
        .success();

    let content =
        std::fs::read_to_string(temp_file.path()).expect("output file should be readable");
    assert!(content.contains("test"));
    assert!(content.contains("word"));
}

//
// Input/Output Testing
//

#[test]
fn test_args_input_from_file() {
    let temp_file = NamedTempFile::new().expect("create temp file");
    fs::write(&temp_file, "test content").expect("test file should be writable");

    let path = temp_file.path().display().to_string();

    Command::cargo_bin("word-tally")
        .expect("arguments should be valid")
        .arg(temp_file.path())
        .arg("-v")
        .assert()
        .success()
        .stderr(contains(format!("source {path}")));
}

#[test]
fn test_args_output_to_file() {
    let temp_file = NamedTempFile::new().expect("create temp file");

    Command::cargo_bin("word-tally")
        .expect("arguments should be valid")
        .arg("--output")
        .arg(temp_file.path())
        .write_stdin("test word test")
        .assert()
        .success();

    let content = fs::read_to_string(temp_file.path()).expect("output file should be readable");
    assert!(content.contains("test 2"));
    assert!(content.contains("word 1"));
}

#[test]
fn test_args_stdin_explicit() {
    Command::cargo_bin("word-tally")
        .expect("arguments should be valid")
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
        .expect("arguments should be valid")
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
        .expect("arguments should be valid")
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
        .expect("arguments should be valid")
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
        .expect("arguments should be valid")
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
        .expect("arguments should be valid")
        .arg("--case=lower")
        .arg("--sort=desc")
        .arg("--io=parallel-in-memory")
        .arg("--format=csv")
        .arg("--delimiter=;")
        .arg("-v")
        .write_stdin("Test TEST test")
        .assert()
        .success()
        .stderr(contains("io,"))
        .stderr(contains("in-memory"));
}

//
// All Enum Mode Testing
//

#[test]
fn test_args_all_io_modes() {
    // Test stream modes with stdin
    let stdin_modes = [
        ("stream", "stream"),
        ("parallel-stream", "parallel-stream"),
        ("parallel-in-memory", "parallel-in-memory"),
    ];

    for (input, expected) in &stdin_modes {
        Command::cargo_bin("word-tally")
            .expect("arguments should be valid")
            .arg(format!("--io={input}"))
            .arg("-v")
            .write_stdin("test")
            .assert()
            .success()
            .stderr(contains(format!("io {expected}")));
    }

    // Test mmap separately - it should fail with stdin
    Command::cargo_bin("word-tally")
        .expect("arguments should be valid")
        .arg("--io=parallel-mmap")
        .write_stdin("test")
        .assert()
        .failure()
        .code(64)
        .stderr(contains("memory-mapped I/O requires a file, not stdin"));
}

#[test]
fn test_args_case_modes() {
    for case in ["original", "upper", "lower"] {
        test_verbose_command(
            &[&format!("--case={case}"), "-v"],
            "Test",
            &[&format!("case {case}")],
        );
    }
}

#[test]
fn test_args_sort_modes() {
    for sort in ["unsorted", "asc", "desc"] {
        test_verbose_command(
            &[&format!("--sort={sort}"), "-v"],
            "test",
            &[&format!("order {sort}")],
        );
    }
}

#[test]
fn test_args_format_modes() {
    for format in ["text", "json", "csv"] {
        test_verbose_command(&[&format!("--format={format}"), "-v"], "test", &["source"]);
    }
}

//
// Edge Cases
//

#[test]
fn test_args_delimiter_edge_cases() {
    // Empty delimiter
    Command::cargo_bin("word-tally")
        .expect("arguments should be valid")
        .arg("--delimiter=")
        .arg("--format=text")
        .write_stdin("test word")
        .assert()
        .success()
        .stdout(contains("test1"))
        .stdout(contains("word1"));

    // Multi-char delimiter
    Command::cargo_bin("word-tally")
        .expect("arguments should be valid")
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
    word_tally()
        .arg("--io=bytes")
        .write_stdin("test")
        .assert()
        .failure()
        .stderr(contains("invalid value 'bytes'"));
}

#[test]
fn test_args_invalid_min_count() {
    word_tally()
        .arg("--min-count=invalid")
        .write_stdin("test")
        .assert()
        .failure()
        .stderr(contains("invalid value"));
}

fn test_invalid_enum_value(arg: &str) {
    word_tally()
        .arg(arg)
        .write_stdin("test")
        .assert()
        .failure()
        .stderr(contains("invalid value"));
}

#[test]
fn test_args_invalid_enum_values() {
    test_invalid_enum_value("--case=invalid");
    test_invalid_enum_value("--sort=invalid");
}

#[test]
fn test_args_invalid_regex_exclude() {
    word_tally()
        .arg("--exclude=[")
        .write_stdin("test")
        .assert()
        .failure()
        .stderr(contains("invalid exclude pattern"));
}

#[test]
fn test_args_invalid_regex_include() {
    word_tally()
        .arg("--include=(?P<")
        .write_stdin("test")
        .assert()
        .failure()
        .stderr(contains("invalid include pattern"));
}

//
// Environment Interaction
//

#[test]
fn test_args_environment_interaction() {
    word_tally()
        .env("WORD_TALLY_TALLY_CAPACITY", "100")
        .arg("-v")
        .write_stdin("test")
        .assert()
        .success()
        .stderr(contains("source"));
}

#[test]
fn test_args_mmap_alias() {
    let temp_file = NamedTempFile::new().expect("create temp file");
    fs::write(temp_file.path(), "test content").expect("write test content");

    word_tally()
        .arg("--io=mmap")
        .arg("-v")
        .arg(temp_file.path())
        .assert()
        .success()
        .stderr(contains("io parallel-mmap"));

    word_tally()
        .arg("--io=mmap")
        .write_stdin("test")
        .assert()
        .failure()
        .code(64)
        .stderr(contains("memory-mapped I/O requires a file, not stdin"));
}
