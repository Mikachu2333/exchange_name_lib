use std::{
    fs,
    path::Path,
    sync::{Mutex, MutexGuard, OnceLock},
};

use tempfile::Builder;

use crate::{plan::ExchangePlan, RenameError};

static OPERATION_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

pub(crate) fn exchange(first: &Path, second: &Path, preserve_ext: bool) -> Result<(), RenameError> {
    let _guard = lock_operations();
    let plan = ExchangePlan::build(first, second, preserve_ext)?;
    execute_with(&plan, |from, to| fs::rename(from, to).map_err(Into::into))
}

fn execute_with(
    plan: &ExchangePlan,
    mut rename: impl FnMut(&Path, &Path) -> Result<(), RenameError>,
) -> Result<(), RenameError> {
    let temp_parent = plan.second.source.parent().ok_or_else(|| {
        RenameError::InvalidPath(format!(
            "path has no parent: {}",
            plan.second.source.display()
        ))
    })?;
    let temp_dir = Builder::new()
        .prefix(".name-exchange-")
        .tempdir_in(temp_parent)
        .map_err(RenameError::from)?
        .keep();
    let temporary = temp_dir.join("entry");
    let result = run_steps(plan, &temporary, &mut rename);
    let _ = fs::remove_dir(temp_dir);
    result
}

fn run_steps(
    plan: &ExchangePlan,
    temporary: &Path,
    rename: &mut impl FnMut(&Path, &Path) -> Result<(), RenameError>,
) -> Result<(), RenameError> {
    let rollback_failed =
        |operation: &RenameError, rollback: &RenameError| RenameError::RollbackFailed {
            operation: operation.to_string(),
            rollback: format!("{rollback}; staged entry: {}", temporary.display()),
        };
    rename(&plan.second.source, temporary)?;
    if let Err(operation) = rename(&plan.first.source, &plan.first.target) {
        restore(temporary, &plan.second.source, rename)
            .map_err(|rollback| rollback_failed(&operation, &rollback))?;
        return Err(operation);
    }

    if let Err(operation) = rename(temporary, &plan.second.target) {
        if plan.first.source != plan.first.target {
            restore(&plan.first.target, &plan.first.source, rename)
                .map_err(|rollback| rollback_failed(&operation, &rollback))?;
        }
        restore(temporary, &plan.second.source, rename)
            .map_err(|rollback| rollback_failed(&operation, &rollback))?;
        return Err(operation);
    }
    Ok(())
}

fn restore(
    from: &Path,
    to: &Path,
    rename: &mut impl FnMut(&Path, &Path) -> Result<(), RenameError>,
) -> Result<(), RenameError> {
    match fs::symlink_metadata(to) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => rename(from, to),
        Err(error) => Err(error.into()),
        Ok(_) => Err(RenameError::AlreadyExists),
    }
}

fn lock_operations() -> MutexGuard<'static, ()> {
    OPERATION_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn failed_renames_preserve_both_entries() {
        for (failures, first_moved, staged) in [
            (&[2][..], false, false),
            (&[3][..], false, false),
            (&[2, 3][..], false, true),
            (&[3, 4][..], true, true),
            (&[3, 5][..], false, true),
        ] {
            let root = tempfile::tempdir().unwrap();
            let first = root.path().join("first");
            let second = root.path().join("second");
            fs::write(&first, b"A").unwrap();
            fs::create_dir(&second).unwrap();
            fs::write(second.join("child"), b"B").unwrap();
            let plan = ExchangePlan::build(&first, &second, false).unwrap();
            let mut temporary = None;
            let mut calls = 0;
            let error = execute_with(&plan, |from, to| {
                calls += 1;
                if calls == 1 {
                    temporary = Some(to.to_owned());
                }
                if failures.contains(&calls) {
                    return Err(RenameError::PermissionDenied);
                }
                fs::rename(from, to).map_err(Into::into)
            })
            .unwrap_err();
            assert_eq!(
                fs::read(if first_moved { &second } else { &first }).unwrap(),
                b"A"
            );
            let temporary = temporary.unwrap();
            if staged {
                assert_eq!(error.to_code(), 7);
                assert!(error
                    .to_string()
                    .contains(temporary.to_string_lossy().as_ref()));
                assert_eq!(fs::read(temporary.join("child")).unwrap(), b"B");
            } else {
                assert_eq!(error, RenameError::PermissionDenied);
                assert!(!temporary.parent().unwrap().exists());
                assert_eq!(fs::read(second.join("child")).unwrap(), b"B");
            }
        }
    }
}
