use std::{fs, path::PathBuf};

use same_file::is_same_file;

use crate::{
    entry::{compose_file_name, Entry, EntryKind},
    resolver::{current_base_dir, resolve},
    RenameError,
};

#[derive(Debug)]
pub(crate) struct RenameStep {
    pub(crate) source: PathBuf,
    pub(crate) target: PathBuf,
}

#[derive(Debug)]
pub(crate) struct ExchangePlan {
    pub(crate) first: RenameStep,
    pub(crate) second: RenameStep,
}

impl ExchangePlan {
    pub(crate) fn build(
        first_path: &std::path::Path,
        second_path: &std::path::Path,
        preserve_ext: bool,
    ) -> Result<Self, RenameError> {
        let base_dir = current_base_dir()?;
        let first_path = resolve(first_path, &base_dir)?.into_existing()?;
        let second_path = resolve(second_path, &base_dir)?.into_existing()?;

        if first_path == second_path
            || is_same_file(&first_path, &second_path).map_err(RenameError::from)?
        {
            return Err(RenameError::SamePath);
        }

        let first = Entry::inspect(first_path)?;
        let second = Entry::inspect(second_path)?;
        reject_nested_directories(&first, &second)?;

        let first_target = target_for(&first, &second, preserve_ext);
        let second_target = target_for(&second, &first, preserve_ext);
        if first_target == second_target {
            return Err(RenameError::AlreadyExists);
        }

        ensure_available(&first_target, &first, &second)?;
        ensure_available(&second_target, &first, &second)?;

        Ok(Self {
            first: RenameStep {
                source: first.path,
                target: first_target,
            },
            second: RenameStep {
                source: second.path,
                target: second_target,
            },
        })
    }
}

fn reject_nested_directories(first: &Entry, second: &Entry) -> Result<(), RenameError> {
    let nested = (first.is_directory() && second.path.starts_with(&first.path))
        || (second.is_directory() && first.path.starts_with(&second.path));
    if nested {
        Err(RenameError::InvalidPath(
            "ancestor/descendant paths cannot be exchanged safely".to_owned(),
        ))
    } else {
        Ok(())
    }
}

fn target_for(entry: &Entry, other: &Entry, preserve_ext: bool) -> PathBuf {
    let name = if entry.kind == EntryKind::File && other.kind == EntryKind::File {
        let extension = if preserve_ext {
            entry.extension.as_deref()
        } else {
            other.extension.as_deref()
        };
        compose_file_name(&other.stem, extension)
    } else {
        other.file_name.clone()
    };
    entry.parent.join(name)
}

fn ensure_available(
    target: &std::path::Path,
    first: &Entry,
    second: &Entry,
) -> Result<(), RenameError> {
    if target == first.path || target == second.path {
        return Ok(());
    }
    match fs::symlink_metadata(target) {
        Ok(_) => Err(RenameError::AlreadyExists),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}
