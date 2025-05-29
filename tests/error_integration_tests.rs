//! Tests for error handling integration.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs::{self, File};
use std::io::Write;
use tempfile::{NamedTempFile, TempDir};

#[test]
fn test_permission_denied_error() {
    // Create a temporary directory for our test
    let temp_dir = TempDir::new().expect("process test");
    let output_path = temp_dir.path().join("test-output.txt");

    // Create the file and make it read-only to trigger an error when trying to write
    {
        File::create(&output_path).expect("process test");
        // On Unix-based systems, make the file read-only
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = fs::metadata(&output_path).expect("process test");
            let mut perms = metadata.permissions();
            perms.set_mode(0o444); // Read-only for all users
            fs::set_permissions(&output_path, perms).expect("process test");
        }
    }

    // Set up the command with verbose output and a specific output file
    // This should cause an error when trying to write to the read-only file
    let assert = Command::cargo_bin("word-tally")
        .expect("execute operation")
        .arg("-v")
        .arg("--format=csv")
        .arg(format!("--output={}", output_path.display()))
        .assert();

    // Check that we got the permission denied error with proper exit code
    // According to errors.rs, permission denied gets code 77 (NOPERM)
    assert
        .failure()
        .code(77) // Permission denied code
        .stderr(predicate::str::contains(
            "Error: failed to create output file:",
        ));

    // Clean up
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = fs::metadata(&output_path).expect("process test");
        let mut perms = metadata.permissions();
        perms.set_mode(0o644); // Make writable again for cleanup
        fs::set_permissions(&output_path, perms).expect("process test");
    }
}

#[test]
fn test_nonexistent_path_error() {
    // Create a path to a nonexistent directory
    let temp_dir = TempDir::new().expect("process test");
    let nonexistent_dir = temp_dir.path().join("does_not_exist");
    let output_path = nonexistent_dir.join("output.txt");

    // This should fail because the directory doesn't exist
    let assert = Command::cargo_bin("word-tally")
        .expect("execute operation")
        .arg("-v")
        .arg("--format=csv")
        .arg(format!("--output={}", output_path.display()))
        .assert();

    // According to errors.rs, "not found" errors get exit code 66 (NOINPUT)
    assert
        .failure()
        .code(66) // No input code (also used for not found errors)
        .stderr(predicate::str::contains(
            "Error: failed to create output file:",
        ));
}

#[test]
fn test_input_file_not_found_error() {
    let assert = Command::cargo_bin("word-tally")
        .expect("execute operation")
        .arg("/nonexistent/file.txt")
        .assert();

    assert.failure().stderr(predicate::str::contains(
        "no such file: /nonexistent/file.txt",
    ));
}

#[test]
fn test_input_file_permission_denied_error() {
    let temp_dir = TempDir::new().expect("process test");
    let file_path = temp_dir.path().join("no_read_permission.txt");

    // Create file with no read permissions
    {
        File::create(&file_path).expect("process test");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = fs::metadata(&file_path).expect("process test");
            let mut perms = metadata.permissions();
            perms.set_mode(0o000); // No permissions
            fs::set_permissions(&file_path, perms).expect("process test");
        }
    }

    let assert = Command::cargo_bin("word-tally")
        .expect("execute operation")
        .arg(&file_path)
        .assert();

    assert.failure().stderr(predicate::str::contains(format!(
        "permission denied: {}",
        file_path.display()
    )));

    // Clean up
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = fs::metadata(&file_path).expect("process test");
        let mut perms = metadata.permissions();
        perms.set_mode(0o644); // Restore permissions for cleanup
        fs::set_permissions(&file_path, perms).expect("process test");
    }
}

#[test]
fn test_new_invalid_utf8_error() {
    let mut file = NamedTempFile::new().expect("create temp file");
    // Write invalid UTF-8 sequence
    file.write_all(&[0xFF, 0xFE, 0xFD])
        .expect("write invalid UTF-8");
    file.flush().expect("flush file");

    let assert = Command::cargo_bin("word-tally")
        .expect("execute operation")
        .arg(file.path())
        .assert();

    assert
        .failure()
        .stderr(predicate::str::contains("invalid UTF-8"));
}
