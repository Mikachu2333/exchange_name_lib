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
fn preserves_emoji_sequences_in_names_and_extensions() {
    let root = TempDir::new().unwrap();
    let folder = root.path().join("📂");
    fs::create_dir(&folder).unwrap();
    let first = folder.join("👩🏽‍💻.📝");
    let second = folder.join("🇨🇳.txt");
    write(&first, "A");
    write(&second, "B");

    exchange_rs(&first, &second, true).unwrap();

    assert_eq!(read(&folder.join("🇨🇳.📝")), "A");
    assert_eq!(read(&folder.join("👩🏽‍💻.txt")), "B");
}

#[cfg(windows)]
#[test]
fn preserves_extension_spelling_on_case_insensitive_volumes() {
    let root = TempDir::new().unwrap();
    let first = root.path().join("alpha.TXT");
    let second = root.path().join("beta.txt");
    write(&first, "A");
    write(&second, "B");
    exchange_rs(&first, &second, true).unwrap();
    assert_eq!(read(&root.path().join("beta.TXT")), "A");
    assert_eq!(read(&root.path().join("alpha.txt")), "B");
    let names: Vec<_> = fs::read_dir(root.path())
        .unwrap()
        .map(|item| item.unwrap().file_name())
        .collect();
    assert!(names.contains(&"beta.TXT".into()));
    assert!(names.contains(&"alpha.txt".into()));
}

#[test]
fn a_third_hard_link_still_counts_as_an_occupied_target() {
    let root = TempDir::new().unwrap();
    let first = root.path().join("alpha.txt");
    let second = root.path().join("beta.log");
    let occupied = root.path().join("beta.txt");
    write(&first, "A");
    write(&second, "B");
    fs::hard_link(&second, &occupied).unwrap();
    assert_eq!(
        exchange_rs(&first, &second, true),
        Err(RenameError::AlreadyExists)
    );
    assert_eq!(read(&occupied), "B");
    assert_eq!(read(&first), "A");
    assert_eq!(read(&second), "B");
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
fn rejects_hard_links_to_the_same_file() {
    let root = TempDir::new().unwrap();
    let first = root.path().join("first");
    let alias = root.path().join("alias");
    write(&first, "original");
    fs::hard_link(&first, &alias).unwrap();
    assert_eq!(
        exchange_rs(&first, &alias, false),
        Err(RenameError::SamePath)
    );
}

#[cfg(any(unix, windows))]
#[test]
fn exchanges_link_entries_even_when_targets_match_or_are_missing() {
    #[cfg(unix)]
    use std::os::unix::fs::symlink;
    #[cfg(windows)]
    use std::os::windows::fs::symlink_file as symlink;

    let root = TempDir::new().unwrap();
    let target = root.path().join("target");
    let first = root.path().join("one");
    let second = root.path().join("two");
    write(&target, "original");
    let created = symlink("target", &first);
    #[cfg(windows)]
    if created
        .as_ref()
        .is_err_and(|error| error.raw_os_error() == Some(1314))
    {
        return;
    }
    created.unwrap();
    symlink(&target, &second).unwrap();
    exchange_rs(&first, &second, false).unwrap();
    assert_eq!(fs::read_link(&first).unwrap(), target);
    assert_eq!(fs::read_link(&second).unwrap(), Path::new("target"));
    let dangling = root.path().join("dangling");
    symlink("absent", &dangling).unwrap();
    exchange_rs(&first, &dangling, false).unwrap();
    assert_eq!(fs::read_link(&first).unwrap(), Path::new("absent"));
    assert_eq!(read(&dangling), "original");
}

#[cfg(unix)]
#[test]
fn rejects_a_fifo_without_waiting_for_a_writer() {
    let root = TempDir::new().unwrap();
    let fifo = root.path().join("fifo");
    let file = root.path().join("file");
    write(&file, "original");
    assert!(std::process::Command::new("mkfifo")
        .arg(&fifo)
        .status()
        .unwrap()
        .success());
    let (sender, receiver) = std::sync::mpsc::channel();
    std::thread::scope(|scope| {
        scope.spawn(|| sender.send(exchange_rs(&fifo, &file, false)).unwrap());
        let result = receiver.recv_timeout(std::time::Duration::from_secs(5));
        if result.is_err() {
            let _writer = fs::File::options()
                .read(true)
                .write(true)
                .open(&fifo)
                .unwrap();
            receiver.recv().unwrap().unwrap_err();
            panic!("identity check waited for a FIFO writer");
        }
        assert!(matches!(
            result.unwrap(),
            Err(RenameError::UnsupportedFileType(_))
        ));
    });
}

#[test]
fn concurrent_swaps_of_the_same_pair_serialize() {
    let root = TempDir::new().unwrap();
    let first = root.path().join("first");
    let second = root.path().join("second");
    write(&first, "A");
    write(&second, "B");
    let start = std::sync::Barrier::new(4);
    std::thread::scope(|scope| {
        for _ in 0..4 {
            scope.spawn(|| {
                start.wait();
                for _ in 0..8 {
                    exchange_rs(&first, &second, false).unwrap();
                }
            });
        }
    });
    assert_eq!(read(&first), "A");
    assert_eq!(read(&second), "B");
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

#[cfg(unix)]
#[test]
fn resolves_parent_components_in_filesystem_order() {
    use exchange_name_lib::resolve_path_rs;
    use std::os::unix::fs::symlink;

    let root = TempDir::new().unwrap();
    let base = root.path();
    fs::create_dir_all(base.join("real/child")).unwrap();
    symlink("real/child", base.join("link")).unwrap();
    write(&base.join("name"), "untouched");
    write(&base.join("real/name"), "selected");
    write(&base.join("real/other"), "other");
    exchange_rs(&base.join("link/../name"), &base.join("real/other"), false).unwrap();
    assert_eq!(read(&base.join("name")), "untouched");
    assert_eq!(read(&base.join("real/other")), "selected");
    assert_eq!(
        resolve_path_rs(Path::new("link/.."), base).unwrap(),
        (true, base.join("real").canonicalize().unwrap())
    );
    assert!(
        !resolve_path_rs(Path::new("missing/../name"), base)
            .unwrap()
            .0
    );
    assert_eq!(
        exchange_rs(
            &base.join("missing/../name"),
            &base.join("real/other"),
            false
        ),
        Err(RenameError::NotExists)
    );
}
