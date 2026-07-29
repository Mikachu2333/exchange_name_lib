#![deny(unsafe_op_in_unsafe_fn)]

use std::path::{Path, PathBuf};

mod entry;
mod error;
mod ffi;
mod plan;
mod resolver;
mod transaction;

pub use error::RenameError;
pub use ffi::{exchange, exchange_n};

/// Swaps names of two files, directories, or symbolic links.
///
/// Each rename is atomic, but the complete exchange is not a crash-safe transaction.
///
/// # Errors
///
/// Returns [`RenameError`] when validation, renaming, or rollback fails.
pub fn exchange_rs(path1: &Path, path2: &Path, preserve_ext: bool) -> Result<(), RenameError> {
    let plan = plan::ExchangePlan::build(path1, path2, preserve_ext)?;
    transaction::execute(&plan)
}

/// Resolves a path without dereferencing its final symbolic-link component.
///
/// # Errors
///
/// Returns [`RenameError`] when expansion, normalization, or metadata access fails.
pub fn resolve_path_rs(path: &Path, base_dir: &Path) -> Result<(bool, PathBuf), RenameError> {
    resolver::resolve(path, base_dir).map(resolver::ResolvedPath::into_legacy_tuple)
}
