use assert_cmd::Command;
use predicates::prelude::*;
use std::fs::{self, File};
use tempfile::TempDir;

#[test]
fn test_permission_denied_error() {
    // Create a temporary directory for our test
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("test-output.txt");

    // Create the file and make it read-only to trigger an error when trying to write
    {
        File::create(&output_path).unwrap();
        // On Unix-based systems, make the file read-only
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = fs::metadata(&output_path).unwrap();
            let mut perms = metadata.permissions();
            perms.set_mode(0o444); // Read-only for all users
            fs::set_permissions(&output_path, perms).unwrap();
        }
    }

    // Set up the command with verbose output and a specific output file
    // This should cause an error when trying to write to the read-only file
    let assert = Command::cargo_bin("word-tally")
        .unwrap()
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
            "Error: Failed to create output file:",
        ));

    // Clean up
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = fs::metadata(&output_path).unwrap();
        let mut perms = metadata.permissions();
        perms.set_mode(0o644); // Make writable again for cleanup
        fs::set_permissions(&output_path, perms).unwrap();
    }
}

#[test]
fn test_nonexistent_path_error() {
    // Create a path to a nonexistent directory
    let temp_dir = TempDir::new().unwrap();
    let nonexistent_dir = temp_dir.path().join("does_not_exist");
    let output_path = nonexistent_dir.join("output.txt");

    // This should fail because the directory doesn't exist
    let assert = Command::cargo_bin("word-tally")
        .unwrap()
        .arg("-v")
        .arg("--format=csv")
        .arg(format!("--output={}", output_path.display()))
        .assert();

    // According to errors.rs, "not found" errors get exit code 66 (NOINPUT)
    assert
        .failure()
        .code(66) // No input code (also used for not found errors)
        .stderr(predicate::str::contains(
            "Error: Failed to create output file:",
        ));
}
