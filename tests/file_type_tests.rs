//! Tests for `FileType` functionality.

use std::collections::HashSet;

use word_tally::FileType;

#[test]
fn file_type_display() {
    assert_eq!(FileType::File.to_string(), "file");
    assert_eq!(FileType::Directory.to_string(), "directory");
    assert_eq!(FileType::NamedPipe.to_string(), "named pipe");
    assert_eq!(FileType::Socket.to_string(), "socket");
    assert_eq!(FileType::CharacterDevice.to_string(), "character device");
    assert_eq!(FileType::BlockDevice.to_string(), "block device");
    assert_eq!(FileType::Special.to_string(), "special file");
}

#[test]
fn file_type_debug() {
    assert_eq!(format!("{:?}", FileType::File), "File");
    assert_eq!(format!("{:?}", FileType::Directory), "Directory");
    assert_eq!(format!("{:?}", FileType::NamedPipe), "NamedPipe");
    assert_eq!(format!("{:?}", FileType::Socket), "Socket");
    assert_eq!(
        format!("{:?}", FileType::CharacterDevice),
        "CharacterDevice"
    );
    assert_eq!(format!("{:?}", FileType::BlockDevice), "BlockDevice");
    assert_eq!(format!("{:?}", FileType::Special), "Special");
}

#[test]
fn file_type_clone() {
    let original = FileType::File;
    let cloned = original;
    assert_eq!(original, cloned);
}

#[test]
fn file_type_copy() {
    let original = FileType::Directory;
    let copied = original;
    assert_eq!(original, copied);
}

#[test]
fn file_type_equality() {
    assert_eq!(FileType::File, FileType::File);
    assert_ne!(FileType::File, FileType::Directory);
    assert_ne!(FileType::Directory, FileType::NamedPipe);
}

#[test]
fn file_type_ordering() {
    let mut types = [
        FileType::Socket,
        FileType::File,
        FileType::Directory,
        FileType::BlockDevice,
        FileType::CharacterDevice,
        FileType::NamedPipe,
        FileType::Special,
    ];
    types.sort();

    assert_eq!(types.len(), 7);
    let types_sorted = types;
    types.sort();
    assert_eq!(types, types_sorted);
}

#[test]
fn file_type_hash() {
    let mut set = HashSet::new();
    set.insert(FileType::File);
    set.insert(FileType::Directory);
    set.insert(FileType::File);
    set.insert(FileType::NamedPipe);
    set.insert(FileType::Socket);
    set.insert(FileType::CharacterDevice);
    set.insert(FileType::BlockDevice);
    set.insert(FileType::Special);

    assert_eq!(set.len(), 7);
    assert!(set.contains(&FileType::File));
    assert!(set.contains(&FileType::Directory));
}

#[test]
fn file_type_from_metadata_regular_file() {
    let temp_file = tempfile::NamedTempFile::new().expect("create temp file");
    let metadata = std::fs::metadata(temp_file.path()).expect("read metadata");
    let file_type = FileType::from(&metadata);
    assert_eq!(file_type, FileType::File);
}

#[test]
fn file_type_from_metadata_directory() {
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let metadata = std::fs::metadata(temp_dir.path()).expect("read metadata");
    let file_type = FileType::from(&metadata);
    assert_eq!(file_type, FileType::Directory);
}

#[cfg(unix)]
#[test]
fn file_type_from_metadata_unix_types() {
    let metadata = std::fs::metadata("/dev/null").expect("read /dev/null metadata");
    let file_type = FileType::from(&metadata);
    assert_eq!(file_type, FileType::CharacterDevice);
}
