use std::{
    fs,
    sync::{Mutex, MutexGuard, OnceLock},
};

use tempfile::Builder;

use crate::{plan::ExchangePlan, RenameError};

static OPERATION_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

pub(crate) fn execute(plan: &ExchangePlan) -> Result<(), RenameError> {
    let _guard = lock_operations();
    let temp_parent = plan.second.source.parent().ok_or_else(|| {
        RenameError::InvalidPath(format!(
            "path has no parent: {}",
            plan.second.source.display()
        ))
    })?;
    let temp_dir = Builder::new()
        .prefix(".name-exchange-")
        .tempdir_in(temp_parent)
        .map_err(RenameError::from)?;
    let temporary = temp_dir.path().join("entry");

    rename(&plan.second.source, &temporary)?;
    if let Err(operation) = rename(&plan.first.source, &plan.first.target) {
        return match rename(&temporary, &plan.second.source) {
            Ok(()) => Err(operation),
            Err(rollback) => Err(rollback_failed(&operation, &rollback)),
        };
    }

    if let Err(operation) = rename(&temporary, &plan.second.target) {
        let first_rollback = rename(&plan.first.target, &plan.first.source);
        let second_rollback = rename(&temporary, &plan.second.source);
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

    // Retrying after a cleanup-only failure would reverse an already successful exchange.
    let _ = temp_dir.close();
    Ok(())
}

fn lock_operations() -> MutexGuard<'static, ()> {
    OPERATION_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn rename(from: &std::path::Path, to: &std::path::Path) -> Result<(), RenameError> {
    fs::rename(from, to).map_err(RenameError::from)
}

fn rollback_failed(operation: &RenameError, rollback: &RenameError) -> RenameError {
    RenameError::RollbackFailed {
        operation: operation.to_string(),
        rollback: rollback.to_string(),
    }
}
