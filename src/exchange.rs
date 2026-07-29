use std::{
    env,
    ffi::OsStr,
    fs,
    path::{Component, Path, PathBuf},
    sync::{Mutex, MutexGuard, OnceLock},
};

use same_file::is_same_file;

use crate::{
    file_rename::swap_paths,
    path_checkout::{compose_file_name, is_strict_ancestor, EntryKind, PathInfo},
    types::RenameError,
};

static OPERATION_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

/// Swap names of two filesystem entries.
pub(crate) fn exchange_paths(
    path1: &Path,
    path2: &Path,
    preserve_ext: bool,
) -> Result<(), RenameError> {
    let _guard = lock_operations();
    let base_dir = resolve_base_dir()?;
    let (exists1, path1) = resolve_path(path1, &base_dir)?;
    let (exists2, path2) = resolve_path(path2, &base_dir)?;
    if !exists1 || !exists2 {
        return Err(RenameError::NotExists);
    }

    if path1 == path2 || is_same_file(&path1, &path2).map_err(RenameError::from)? {
        return Err(RenameError::SamePath);
    }

    let first = PathInfo::inspect(path1)?;
    let second = PathInfo::inspect(path2)?;

    // A parent move invalidates the child's path halfway through a multi-step exchange.
    // Reject this case rather than risk moving entries to unintended locations.
    if (first.is_directory() && is_strict_ancestor(&first.original, &second.original))
        || (second.is_directory() && is_strict_ancestor(&second.original, &first.original))
    {
        return Err(RenameError::InvalidPath(
            "ancestor/descendant paths cannot be exchanged safely".to_owned(),
        ));
    }

    let first_target = target_for(&first, &second, preserve_ext);
    let second_target = target_for(&second, &first, preserve_ext);
    if first_target == second_target {
        return Err(RenameError::AlreadyExists);
    }

    ensure_target_available(&first_target, &first, &second)?;
    ensure_target_available(&second_target, &first, &second)?;

    swap_paths(
        &first.original,
        &first_target,
        &second.original,
        &second_target,
    )
}

fn lock_operations() -> MutexGuard<'static, ()> {
    OPERATION_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn target_for(entry: &PathInfo, other: &PathInfo, preserve_ext: bool) -> PathBuf {
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

fn ensure_target_available(
    target: &Path,
    first: &PathInfo,
    second: &PathInfo,
) -> Result<(), RenameError> {
    if target == first.original || target == second.original {
        return Ok(());
    }
    match fs::symlink_metadata(target) {
        Ok(_) => Err(RenameError::AlreadyExists),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

fn resolve_base_dir() -> Result<PathBuf, RenameError> {
    env::current_dir().map_err(RenameError::from)
}

/// Resolve a path without dereferencing its final component.
///
/// Existing parent directories are canonicalized, but a final symbolic link remains a link.
pub(crate) fn resolve_path(path: &Path, base_dir: &Path) -> Result<(bool, PathBuf), RenameError> {
    if path.as_os_str().is_empty() {
        return Ok((false, path.to_path_buf()));
    }

    let expanded = expand_home(path)?;
    let absolute = if expanded.is_absolute() {
        expanded
    } else {
        base_dir.join(expanded)
    };
    let normalized = normalize_lexically(&absolute)?;

    let resolved = if let (Some(parent), Some(name)) = (normalized.parent(), normalized.file_name())
    {
        match parent.canonicalize() {
            Ok(parent) => parent.join(name),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => normalized,
            Err(error) => return Err(error.into()),
        }
    } else {
        normalized
    };

    match fs::symlink_metadata(&resolved) {
        Ok(_) => Ok((true, resolved)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok((false, resolved)),
        Err(error) => Err(error.into()),
    }
}

fn expand_home(path: &Path) -> Result<PathBuf, RenameError> {
    let mut components = path.components();
    if !matches!(components.next(), Some(Component::Normal(component)) if component == OsStr::new("~"))
    {
        return Ok(path.to_path_buf());
    }

    let variable = if cfg!(windows) { "USERPROFILE" } else { "HOME" };
    let home = env::var_os(variable).ok_or_else(|| {
        RenameError::InvalidPath(format!("{variable} is not set; cannot expand '~'"))
    })?;
    let mut expanded = PathBuf::from(home);
    expanded.extend(components);
    Ok(expanded)
}

fn normalize_lexically(path: &Path) -> Result<PathBuf, RenameError> {
    let mut result = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                if !result.pop() {
                    return Err(RenameError::InvalidPath(format!(
                        "path escapes its root: {}",
                        path.display()
                    )));
                }
            }
            other => result.push(other.as_os_str()),
        }
    }
    Ok(result)
}
