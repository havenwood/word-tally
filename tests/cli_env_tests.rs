use assert_cmd::Command;
use predicates::str::contains;

#[test]
fn test_word_tally_chunk_size_env() {
    // Test WORD_TALLY_CHUNK_SIZE
    let assert = Command::cargo_bin("word-tally")
        .expect("execute operation")
        .env("WORD_TALLY_CHUNK_SIZE", "131072") // 128KB
        .arg("--verbose")
        .write_stdin("hope forever")
        .assert();

    assert.success().stderr(contains("source"));
}

#[test]
fn test_word_tally_stdin_buffer_size_env() {
    // Test WORD_TALLY_STDIN_BUFFER_SIZE
    let assert = Command::cargo_bin("word-tally")
        .expect("execute operation")
        .env("WORD_TALLY_STDIN_BUFFER_SIZE", "524288") // 512KB
        .arg("--verbose")
        .write_stdin("hope forever")
        .assert();

    assert.success().stderr(contains("source"));
}

#[test]
fn test_word_tally_uniqueness_ratio_env() {
    // Test WORD_TALLY_UNIQUENESS_RATIO
    let assert = Command::cargo_bin("word-tally")
        .expect("execute operation")
        .env("WORD_TALLY_UNIQUENESS_RATIO", "20")
        .arg("--verbose")
        .write_stdin("hope forever")
        .assert();

    assert.success().stderr(contains("source"));
}

#[test]
fn test_word_tally_words_per_kb_env() {
    // Test WORD_TALLY_WORDS_PER_KB
    let assert = Command::cargo_bin("word-tally")
        .expect("execute operation")
        .env("WORD_TALLY_WORDS_PER_KB", "250")
        .arg("--verbose")
        .write_stdin("hope forever")
        .assert();

    assert.success().stderr(contains("source"));
}
