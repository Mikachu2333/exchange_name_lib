use std::{
    env,
    path::{Path, PathBuf},
};

use crate::types::{GetPathInfo, NameExchange, RenameError};

pub fn exchange_paths(path1: PathBuf, path2: PathBuf) -> Result<(), RenameError> {
    let base_dir = resolve_base_dir()?;

    let (path1, exists1) = resolve_path(path1, &base_dir);
    let (path2, exists2) = resolve_path(path2, &base_dir);

    if !exists1 || !exists2 {
        return Err(RenameError::NotExists);
    }

    if path1 == path2 {
        return Err(RenameError::NotExists);
    }

    let mut exchange_info = NameExchange::new();
    exchange_info.f1.is_exist = true;
    exchange_info.f2.is_exist = true;

    let original_paths = GetPathInfo { path1, path2 };

    (exchange_info.f1.is_file, exchange_info.f2.is_file) = original_paths.if_file();
    (exchange_info.f1.packed_info, exchange_info.f2.packed_info) =
        original_paths.metadata_collect(exchange_info.f1.is_file, exchange_info.f2.is_file);

    exchange_info.f1.exchange.original_path = original_paths.path1.clone();
    exchange_info.f2.exchange.original_path = original_paths.path2.clone();

    (
        exchange_info.f1.exchange.pre_path,
        exchange_info.f1.exchange.new_path,
    ) = NameExchange::make_name(
        &exchange_info.f1.packed_info.parent_dir,
        &exchange_info.f2.packed_info.name,
        &exchange_info.f1.packed_info.ext,
    );
    (
        exchange_info.f2.exchange.pre_path,
        exchange_info.f2.exchange.new_path,
    ) = NameExchange::make_name(
        &exchange_info.f2.packed_info.parent_dir,
        &exchange_info.f1.packed_info.name,
        &exchange_info.f2.packed_info.ext,
    );

    let new_path_conflict_1 = exchange_info.f1.exchange.new_path.exists();
    let new_path_conflict_2 = exchange_info.f2.exchange.new_path.exists();
    let same_parent = GetPathInfo {
        path1: exchange_info.f1.exchange.new_path.clone(),
        path2: exchange_info.f2.exchange.new_path.clone(),
    }
    .if_same_dir();

    if !same_parent && (new_path_conflict_1 || new_path_conflict_2) {
        return Err(RenameError::AlreadyExists);
    }

    let mode = original_paths.if_root();

    match (exchange_info.f1.is_file, exchange_info.f2.is_file) {
        (true, true) => NameExchange::rename_each(&exchange_info, false, true),
        (false, false) => match mode {
            1 => NameExchange::rename_each(&exchange_info, true, false),
            2 => NameExchange::rename_each(&exchange_info, true, true),
            _ => NameExchange::rename_each(&exchange_info, false, true),
        },
        (true, false) => {
            if mode == 2 {
                NameExchange::rename_each(&exchange_info, true, true)
            } else {
                NameExchange::rename_each(&exchange_info, false, true)
            }
        }
        (false, true) => {
            if mode == 1 {
                NameExchange::rename_each(&exchange_info, true, false)
            } else {
                NameExchange::rename_each(&exchange_info, false, false)
            }
        }
    }
}

fn resolve_base_dir() -> Result<PathBuf, RenameError> {
    if let Ok(exe) = env::current_exe() {
        if let Some(parent) = exe.parent() {
            return Ok(parent.to_path_buf());
        }
    }

    env::current_dir().map_err(|err| {
        RenameError::Unknown(format!("Failed to resolve working directory: {}", err))
    })
}

fn resolve_path(mut path: PathBuf, base_dir: &Path) -> (PathBuf, bool) {
    if path.as_os_str().is_empty() {
        return (path, false);
    }

    if !path.is_absolute() {
        path = base_dir.join(path);
    }

    let exists = path.exists();
    if exists {
        if let Ok(canonical) = path.canonicalize() {
            return (canonical, true);
        }
    }

    (path, exists)
}
