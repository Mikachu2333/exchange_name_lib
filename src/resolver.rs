use std::{
    env,
    ffi::OsStr,
    fs,
    path::{Component, Path, PathBuf},
};

use crate::RenameError;

pub(crate) enum ResolvedPath {
    Existing(PathBuf),
    Missing(PathBuf),
}

impl ResolvedPath {
    pub(crate) fn into_existing(self) -> Result<PathBuf, RenameError> {
        match self {
            Self::Existing(path) => Ok(path),
            Self::Missing(_) => Err(RenameError::NotExists),
        }
    }

    pub(crate) fn into_legacy_tuple(self) -> (bool, PathBuf) {
        match self {
            Self::Existing(path) => (true, path),
            Self::Missing(path) => (false, path),
        }
    }
}

pub(crate) fn current_base_dir() -> Result<PathBuf, RenameError> {
    env::current_dir().map_err(RenameError::from)
}

/// Resolves parents while preserving a symbolic link in the final component.
pub(crate) fn resolve(path: &Path, base_dir: &Path) -> Result<ResolvedPath, RenameError> {
    if path.as_os_str().is_empty() {
        return Ok(ResolvedPath::Missing(path.to_path_buf()));
    }

    let expanded = expand_home(path)?;
    let absolute = if expanded.is_absolute() {
        expanded
    } else {
        base_dir.join(expanded)
    };
    let normalized = normalize_lexically(&absolute)?;
    let resolved = resolve_parent(normalized)?;

    match fs::symlink_metadata(&resolved) {
        Ok(_) => Ok(ResolvedPath::Existing(resolved)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            Ok(ResolvedPath::Missing(resolved))
        }
        Err(error) => Err(error.into()),
    }
}

fn resolve_parent(path: PathBuf) -> Result<PathBuf, RenameError> {
    let (Some(parent), Some(name)) = (path.parent(), path.file_name()) else {
        return Ok(path);
    };
    match parent.canonicalize() {
        Ok(parent) => Ok(parent.join(name)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(path),
        Err(error) => Err(error.into()),
    }
}

fn expand_home(path: &Path) -> Result<PathBuf, RenameError> {
    let mut components = path.components();
    if !matches!(components.next(), Some(Component::Normal(part)) if part == OsStr::new("~")) {
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
            Component::ParentDir if !result.pop() => {
                return Err(RenameError::InvalidPath(format!(
                    "path escapes its root: {}",
                    path.display()
                )));
            }
            Component::CurDir | Component::ParentDir => {}
            other => result.push(other.as_os_str()),
        }
    }
    Ok(result)
}
