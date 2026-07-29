use std::{fs, path::Path};

use tempfile::Builder;

use crate::types::RenameError;

/// Perform a three-step exchange with best-effort rollback.
///
/// Each individual rename is atomic on a single filesystem. The complete exchange is not a
/// filesystem transaction: another process can observe intermediate states, and a process crash
/// can leave the temporary directory behind.
pub(crate) fn swap_paths(
    first_source: &Path,
    first_target: &Path,
    second_source: &Path,
    second_target: &Path,
) -> Result<(), RenameError> {
    let temp_parent = second_source.parent().ok_or_else(|| {
        RenameError::InvalidPath(format!("path has no parent: {}", second_source.display()))
    })?;
    let temp_dir = Builder::new()
        .prefix(".name-exchange-")
        .tempdir_in(temp_parent)
        .map_err(RenameError::from)?;
    let temporary = temp_dir.path().join("entry");

    rename(second_source, &temporary)?;

    if let Err(operation) = rename(first_source, first_target) {
        return match rename(&temporary, second_source) {
            Ok(()) => Err(operation),
            Err(rollback) => Err(rollback_failed(&operation, &rollback)),
        };
    }

    if let Err(operation) = rename(&temporary, second_target) {
        let first_rollback = rename(first_target, first_source);
        let second_rollback = rename(&temporary, second_source);
        return match (first_rollback, second_rollback) {
            (Ok(()), Ok(())) => Err(operation),
            (first_result, second_result) => {
                let rollback = [first_result.err(), second_result.err()]
                    .into_iter()
                    .flatten()
                    .map(|error| error.to_string())
                    .collect::<Vec<_>>()
                    .join("; ");
                Err(RenameError::RollbackFailed {
                    operation: operation.to_string(),
                    rollback,
                })
            }
        };
    }

    // The exchange has completed; failure to remove an empty private directory must not report
    // the exchange itself as failed, because retrying would reverse the successful operation.
    let _ = temp_dir.close();
    Ok(())
}

fn rename(from: &Path, to: &Path) -> Result<(), RenameError> {
    // Windows refuses to replace an existing destination. On Unix, callers check for conflicts;
    // an unrelated process can still race this operation because portable Rust has no
    // no-replace rename primitive for every supported Unix platform.
    fs::rename(from, to).map_err(RenameError::from)
}

fn rollback_failed(operation: &RenameError, rollback: &RenameError) -> RenameError {
    RenameError::RollbackFailed {
        operation: operation.to_string(),
        rollback: rollback.to_string(),
    }
}
