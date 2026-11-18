use std::{
    ffi::{OsStr, OsString},
    fs,
    path::{Path, PathBuf},
};

use crate::RenameError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EntryKind {
    File,
    Directory,
    Symlink,
}

#[derive(Debug)]
pub(crate) struct Entry {
    pub(crate) path: PathBuf,
    pub(crate) parent: PathBuf,
    pub(crate) file_name: OsString,
    pub(crate) stem: OsString,
    pub(crate) extension: Option<OsString>,
    pub(crate) kind: EntryKind,
    #[cfg(unix)]
    pub(crate) identity: (u64, u64),
    #[cfg(not(unix))]
    pub(crate) identity: same_file::Handle,
}

impl Entry {
    pub(crate) fn inspect(path: PathBuf, metadata: &fs::Metadata) -> Result<Self, RenameError> {
        let file_type = metadata.file_type();
        let kind = if file_type.is_symlink() {
            EntryKind::Symlink
        } else if file_type.is_file() {
            EntryKind::File
        } else if file_type.is_dir() {
            EntryKind::Directory
        } else {
            return Err(RenameError::UnsupportedFileType(path));
        };

        let parent = path
            .parent()
            .ok_or_else(|| invalid_component(&path, "parent"))?
            .to_path_buf();
        let file_name = path
            .file_name()
            .ok_or_else(|| invalid_component(&path, "file name"))?
            .to_os_string();
        let (stem, extension) = if kind == EntryKind::File {
            split_file_name(&file_name)
        } else {
            (file_name.clone(), None)
        };

        #[cfg(unix)]
        let identity = {
            use std::os::unix::fs::MetadataExt;
            (metadata.dev(), metadata.ino())
        };
        #[cfg(windows)]
        let identity = {
            use std::os::windows::fs::OpenOptionsExt;
            use windows_sys::Win32::Storage::FileSystem::{
                FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT, FILE_READ_ATTRIBUTES,
            };
            let file = fs::File::options()
                .access_mode(FILE_READ_ATTRIBUTES)
                .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
                .open(&path)?;
            same_file::Handle::from_file(file)?
        };
        #[cfg(not(any(unix, windows)))]
        let identity = same_file::Handle::from_path(&path)?;
        Ok(Self {
            path,
            parent,
            file_name,
            stem,
            extension,
            kind,
            identity,
        })
    }

    pub(crate) fn is_directory(&self) -> bool {
        self.kind == EntryKind::Directory
    }
}

fn invalid_component(path: &Path, component: &str) -> RenameError {
    RenameError::InvalidPath(format!("path has no {component}: {}", path.display()))
}

fn split_file_name(file_name: &OsStr) -> (OsString, Option<OsString>) {
    let path = Path::new(file_name);
    (
        path.file_stem().unwrap_or(file_name).to_os_string(),
        path.extension().map(OsStr::to_os_string),
    )
}

pub(crate) fn compose_file_name(stem: &OsStr, extension: Option<&OsStr>) -> OsString {
    let mut name = stem.to_os_string();
    if let Some(extension) = extension {
        name.push(".");
        name.push(extension);
    }
    name
}
