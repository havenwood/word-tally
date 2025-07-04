//! Tests for command-line argument parsing.

use std::fs;

use assert_cmd::Command;
use predicates::str::contains;
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

//
// Default Values Testing
//

#[test]
fn args_defaults() {
    word_tally()
        .arg("-v")
        .write_stdin("test")
        .assert()
        .success()
        .stderr(contains("source -"))
        .stderr(contains("io auto"))
        .stderr(contains("case original"))
        .stderr(contains("order desc"))
        .stderr(contains("delimiter \" \""))
        .stderr(contains("min-chars none"))
        .stderr(contains("min-count none"))
        .stderr(contains("exclude-words none"))
        .stderr(contains("exclude-patterns none"))
        .stderr(contains("include-patterns none"));
}

#[test]
fn args_default_verbose() {
    word_tally()
        .write_stdin("test")
        .assert()
        .success()
        .stdout(contains("test 1"))
        .stderr(predicates::str::is_empty());
}

#[test]
fn args_default_output() {
    word_tally()
        .write_stdin("test")
        .assert()
        .success()
        .stdout(contains("test 1"));
}

#[test]
fn args_all_defaults() {
    word_tally()
        .write_stdin("This is a test")
        .assert()
        .success()
        .stdout(contains("This 1"))
        .stdout(contains("is 1"))
        .stdout(contains("a 1"))
        .stdout(contains("test 1"));
}

#[test]
fn args_defaults_with_minimal_flags() {
    word_tally()
        .arg("-v")
        .write_stdin("Test")
        .assert()
        .success()
        .stdout(contains("Test 1"))
        .stderr(contains("source -"))
        .stderr(contains("total-words 1"))
        .stderr(contains("unique-words 1"));
}

//
// Basic Argument Parsing
//

#[test]
fn get_options() {
    word_tally()
        .args([
            "--case=upper",
            "--sort=asc",
            "--format=json",
            "--min-chars=3",
            "-v",
        ])
        .write_stdin("hello world")
        .assert()
        .success()
        .stderr(contains("\"case\":\"upper\""))
        .stderr(contains("\"order\":\"asc\""))
        .stderr(contains("\"minChars\":3"));
}

#[test]
fn get_performance() {
    word_tally()
        .arg("-v")
        .write_stdin("hello world")
        .assert()
        .success();
}

#[test]
fn get_formatting() {
    word_tally()
        .args(["--case=upper", "--sort=asc", "--format=json", "-v"])
        .write_stdin("hello")
        .assert()
        .success()
        .stderr(contains("\"case\":\"upper\""))
        .stderr(contains("\"order\":\"asc\""));
}

#[test]
fn get_filters() {
    word_tally()
        .args([
            "--min-chars=3",
            "--min-count=2",
            "--exclude-words=hello,world",
            "-v",
        ])
        .write_stdin("hello world")
        .assert()
        .success()
        .stderr(contains("min-chars 3"))
        .stderr(contains("min-count 2"))
        .stderr(contains("exclude-words hello,world"));
}

#[test]
fn get_filters_with_patterns() {
    word_tally()
        .args(["--include=^h", "--exclude=\\d+", "-v"])
        .write_stdin("hello 123 world")
        .assert()
        .success()
        .stderr(contains("exclude-patterns"));
}

//
// Shorthand Flags Testing
//

#[test]
fn io_shorthand_flag() {
    word_tally()
        .args(["-I=parallel-in-memory", "-v"])
        .write_stdin("hello world")
        .assert()
        .success()
        .stderr(contains("io parallel-in-memory"));
}

#[test]
fn args_shorthand_flags() {
    word_tally()
        .args(["-c=upper", "-s=asc", "-f=json", "-m=3", "-n=2", "-v"])
        .write_stdin("hello world test")
        .assert()
        .success()
        .stderr(contains("\"case\":\"upper\""))
        .stderr(contains("\"order\":\"asc\""))
        .stderr(contains("\"minChars\":3"))
        .stderr(contains("\"minCount\":2"));
}

#[test]
fn args_exclude_words_list() {
    word_tally()
        .arg("-w=the,a,an,and,or")
        .arg("-v")
        .write_stdin("the quick brown fox and a lazy dog")
        .assert()
        .success()
        .stderr(contains("exclude-words the,a,an,and,or"));
}

#[test]
fn args_multiple_include_patterns() {
    word_tally()
        .arg("-i=^h")
        .arg("-i=^w")
        .arg("-v")
        .write_stdin("hello world test")
        .assert()
        .success()
        .stderr(contains("include-patterns ^h,^w"));
}

#[test]
fn args_multiple_exclude_patterns() {
    word_tally()
        .arg("-x=ing$")
        .arg("-x=ed$")
        .arg("-v")
        .write_stdin("testing tested test")
        .assert()
        .success()
        .stderr(contains("exclude-patterns ing$,ed$"));
}

#[test]
fn args_output_file_shorthand() {
    let temp_file = NamedTempFile::new().expect("create temp file");

    word_tally()
        .arg("-o")
        .arg(temp_file.path())
        .write_stdin("test word")
        .assert()
        .success();

    let content = fs::read_to_string(temp_file.path()).expect("output file should be readable");
    assert!(content.contains("test"));
    assert!(content.contains("word"));
}

//
// Input/Output Testing
//

#[test]
fn args_input_from_file() {
    let temp_file = NamedTempFile::new().expect("create temp file");
    fs::write(&temp_file, "test content").expect("test file should be writable");

    let path = temp_file.path().display().to_string();

    word_tally()
        .arg(temp_file.path())
        .arg("-v")
        .assert()
        .success()
        .stderr(contains(format!("source {path}")));
}

#[test]
fn args_output_to_file() {
    let temp_file = NamedTempFile::new().expect("create temp file");

    word_tally()
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
fn args_stdin_explicit() {
    word_tally()
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
fn args_verbose_flag() {
    word_tally()
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
fn args_filters_comprehensive() {
    word_tally()
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
fn args_escaped_words() {
    word_tally()
        .arg("--exclude-words=\\t,\\n")
        .arg("-v")
        .write_stdin("test\ttab\nnewline")
        .assert()
        .success()
        .stderr(contains("exclude-words"));
}

#[test]
fn args_multiple_patterns() {
    word_tally()
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
fn args_options_comprehensive() {
    word_tally()
        .arg("--case=lower")
        .arg("--sort=desc")
        .arg("--io=parallel-in-memory")
        .arg("--format=csv")
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
fn args_all_io_modes() {
    // Test valid stdin modes
    word_tally()
        .arg("--io=stream")
        .write_stdin("test")
        .assert()
        .success()
        .stdout(contains("test 1"));

    word_tally()
        .arg("--io=parallel-stream")
        .write_stdin("test")
        .assert()
        .success()
        .stdout(contains("test 1"));

    // Test mmap fails with stdin
    word_tally()
        .arg("--io=parallel-mmap")
        .write_stdin("test")
        .assert()
        .failure()
        .code(64)
        .stderr(contains("cannot memory map stdin"));
}

#[test]
fn args_case_modes() {
    // Test original case
    word_tally()
        .arg("--case=original")
        .write_stdin("Test")
        .assert()
        .success()
        .stdout(contains("Test 1"));

    // Test upper case
    word_tally()
        .arg("--case=upper")
        .write_stdin("Test")
        .assert()
        .success()
        .stdout(contains("TEST 1"));

    // Test lower case
    word_tally()
        .arg("--case=lower")
        .write_stdin("Test")
        .assert()
        .success()
        .stdout(contains("test 1"));
}

#[test]
fn args_sort_modes() {
    for sort in ["unsorted", "asc", "desc"] {
        test_verbose_command(
            &[&format!("--sort={sort}"), "-v"],
            "test",
            &[&format!("order {sort}")],
        );
    }
}

#[test]
fn args_format_modes() {
    for format in ["text", "json", "csv"] {
        test_verbose_command(&[&format!("--format={format}"), "-v"], "test", &["source"]);
    }
}

//
// Edge Cases
//

#[test]
fn args_delimiter_edge_cases() {
    // Test empty delimiter - both words should appear with count 1
    let output = word_tally()
        .arg("--field-delimiter=")
        .write_stdin("test word")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8_lossy(&output);
    assert!(output_str.contains("test1"));
    assert!(output_str.contains("word1"));

    // Test multi-char delimiter
    word_tally()
        .arg("--field-delimiter= => ")
        .write_stdin("test")
        .assert()
        .success()
        .stdout("test => 1\n");
}

//
// Error Handling
//

#[test]
fn args_io_bytes_error() {
    word_tally()
        .arg("--io=bytes")
        .write_stdin("test")
        .assert()
        .failure()
        .stderr(contains("invalid value 'bytes'"));
}

#[test]
fn args_invalid_min_count() {
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
fn args_invalid_enum_values() {
    test_invalid_enum_value("--case=invalid");
    test_invalid_enum_value("--sort=invalid");
}

#[test]
fn args_invalid_regex_exclude() {
    word_tally()
        .arg("--exclude=[")
        .write_stdin("test")
        .assert()
        .failure()
        .stderr(contains("invalid exclude pattern"));
}

#[test]
fn args_invalid_regex_include() {
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
fn args_environment_interaction() {
    word_tally()
        .env("WORD_TALLY_TALLY_CAPACITY", "100")
        .arg("-v")
        .write_stdin("test")
        .assert()
        .success()
        .stderr(contains("source"));
}

#[test]
fn args_mmap_alias() {
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
        .code(64) // EX_USAGE
        .stderr(contains("cannot memory map stdin"));
}

//
// Delimiter Escape Sequence Tests
//

#[test]
fn args_delimiter_unescape() {
    // Test tab delimiter
    word_tally()
        .arg("--field-delimiter=\\t")
        .write_stdin("hello world hello")
        .assert()
        .success()
        .stdout("hello\t2\nworld\t1\n");

    // Test combined delimiters
    word_tally()
        .arg("--field-delimiter=\\t")
        .arg("--entry-delimiter=\\r\\n")
        .write_stdin("test")
        .assert()
        .success()
        .stdout("test\t1\r\n");
}

#[test]
fn args_delimiter_escape_sequences() {
    // Test common escape sequences
    let output_tab = word_tally()
        .arg("--field-delimiter=\\t")
        .write_stdin("word")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    assert!(
        String::from_utf8(output_tab)
            .expect("output should be valid UTF-8")
            .contains("word\t1")
    );

    // Test backslash escape
    let output_backslash = word_tally()
        .arg("--field-delimiter=\\\\")
        .write_stdin("word")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    assert!(
        String::from_utf8(output_backslash)
            .expect("output should be valid UTF-8")
            .contains("word\\1")
    );

    // Test newline escape
    let output_newline = word_tally()
        .arg("--field-delimiter=\\n")
        .write_stdin("word")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    assert!(
        String::from_utf8(output_newline)
            .expect("output should be valid UTF-8")
            .contains("word\n1")
    );
}

#[test]
fn args_delimiter_unknown_escape() {
    // Unknown escape sequences should just use the character
    word_tally()
        .arg("--field-delimiter=\\x")
        .write_stdin("test")
        .assert()
        .success()
        .stdout("testx1\n");
}

#[test]
fn args_delimiter_trailing_backslash() {
    // Trailing backslash is kept as literal
    word_tally()
        .arg("--field-delimiter=\\")
        .write_stdin("test")
        .assert()
        .success()
        .stdout("test\\1\n");
}
