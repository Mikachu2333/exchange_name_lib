use std::{fs, path::Path};

use exchange_name_lib::{exchange_rs, RenameError};
use tempfile::TempDir;

fn write(path: &Path, value: &str) {
    fs::write(path, value).expect("write test file");
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).expect("read test file")
}

#[test]
fn swaps_full_file_names() {
    let dir = TempDir::new().expect("create temp dir");
    let first = dir.path().join("alpha.ext1");
    let second = dir.path().join("beta.ext2");
    write(&first, "A");
    write(&second, "B");

    exchange_rs(&first, &second, false).expect("exchange files");

    assert_eq!(read(&first), "B");
    assert_eq!(read(&second), "A");
}

#[test]
fn preserves_file_extensions() {
    let dir = TempDir::new().expect("create temp dir");
    let first = dir.path().join("alpha.ext1");
    let second = dir.path().join("beta.ext2");
    write(&first, "A");
    write(&second, "B");

    exchange_rs(&first, &second, true).expect("exchange files");

    assert_eq!(read(&dir.path().join("beta.ext1")), "A");
    assert_eq!(read(&dir.path().join("alpha.ext2")), "B");
}

#[test]
fn preserves_spaces_and_quotes_in_names() {
    let dir = TempDir::new().expect("create temp dir");
    let first = dir.path().join("  'alpha'.txt");
    let second = dir.path().join("beta name.log");
    write(&first, "A");
    write(&second, "B");

    exchange_rs(&first, &second, false).expect("exchange files");

    assert_eq!(read(&first), "B");
    assert_eq!(read(&second), "A");
}

#[test]
fn exchanges_file_and_directory_using_complete_names() {
    let dir = TempDir::new().expect("create temp dir");
    let file = dir.path().join("alpha.txt");
    let directory = dir.path().join("beta.dir");
    write(&file, "A");
    fs::create_dir(&directory).expect("create directory");
    write(&directory.join("inside"), "B");

    exchange_rs(&file, &directory, true).expect("exchange entries");

    assert_eq!(read(&directory), "A");
    assert_eq!(read(&file.join("inside")), "B");
}

#[test]
fn swaps_complete_directory_names() {
    let dir = TempDir::new().expect("create temp dir");
    let first = dir.path().join("alpha.dir");
    let second = dir.path().join("beta.folder");
    fs::create_dir(&first).expect("create first dir");
    fs::create_dir(&second).expect("create second dir");
    write(&first.join("first"), "A");
    write(&second.join("second"), "B");

    exchange_rs(&first, &second, true).expect("exchange dirs");

    assert_eq!(read(&first.join("second")), "B");
    assert_eq!(read(&second.join("first")), "A");
}

#[test]
fn rejects_same_file() {
    let dir = TempDir::new().expect("create temp dir");
    let file = dir.path().join("same.ext");
    write(&file, "X");

    assert_eq!(exchange_rs(&file, &file, true), Err(RenameError::SamePath));
}

#[test]
fn rejects_nested_directories_without_mutation() {
    let dir = TempDir::new().expect("create temp dir");
    let parent = dir.path().join("parent");
    let child = parent.join("child");
    fs::create_dir_all(&child).expect("create nested dirs");

    assert!(matches!(
        exchange_rs(&parent, &child, false),
        Err(RenameError::InvalidPath(_))
    ));
    assert!(child.is_dir());
}
