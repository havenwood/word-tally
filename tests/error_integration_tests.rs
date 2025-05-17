use assert_cmd::Command;
use predicates::prelude::*;
use std::fs::{self, File};
use std::io::Write;
use std::sync::Arc;
use tempfile::{NamedTempFile, TempDir};
use word_tally::{Input, Io, Options, WordTally};

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
            "Error: failed to create output file:",
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
            "Error: failed to create output file:",
        ));
}

fn make_shared<T>(value: T) -> Arc<T> {
    Arc::new(value)
}

#[test]
fn test_new_invalid_utf8_error() {
    let invalid_utf8 = vec![0xFF, 0xFE, 0xFD, 0x80, 0x81];
    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(&invalid_utf8).unwrap();

    let options = make_shared(Options::default());
    let input = Input::new(temp_file.path().to_str().unwrap(), options.io()).unwrap();

    let result = WordTally::new(&input, &options);
    assert!(result.is_err());

    let error = result.unwrap_err();
    let error_msg = error.to_string();
    assert!(!error_msg.is_empty());
}

#[test]
#[ignore] // This test hangs waiting for stdin, so we ignore it during regular test runs
fn test_memory_mapping_stdin_error() {
    let options = Options::default().with_io(Io::MemoryMapped);
    let options = make_shared(options);

    // Create stdin input
    let result = {
        // Create an Input that uses stdin
        let input = Input::default();
        WordTally::new(&input, &options)
    };

    // At this point, we should either have an error (if memory mapping stdin is not supported)
    // or success (if it is supported but with no input, giving count 0)
    assert!(result.is_err() || result.as_ref().unwrap().count() == 0);

    if let Err(error) = result {
        let error_msg = error.to_string().to_lowercase();
        assert!(
            error_msg.contains("stdin")
                || error_msg.contains("memory")
                || error_msg.contains("map")
        );
    }
}
