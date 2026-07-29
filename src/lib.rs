#![deny(unsafe_op_in_unsafe_fn)]

use std::{
    ffi::{c_char, CStr},
    panic::{catch_unwind, AssertUnwindSafe},
    path::{Path, PathBuf},
    slice, str,
};

mod exchange;
mod file_rename;
mod path_checkout;
mod types;

use crate::exchange::{exchange_paths, resolve_path};
pub use crate::types::RenameError;

/// Swap names using two NUL-terminated UTF-8 C strings.
///
/// Prefer [`exchange_n`] when the caller can provide explicit buffer lengths.
///
/// # Safety
///
/// Each pointer must be non-null and point to a readable, NUL-terminated byte string that remains
/// valid and immutable for the duration of this call. Reading past an allocation while searching
/// for a terminator is undefined behavior and cannot be detected by this legacy ABI.
///
/// # Return values
///
/// - `0`: success
/// - `1`: path does not exist
/// - `2`: permission denied or read-only filesystem
/// - `3`: target already exists
/// - `4`: both paths refer to the same entry
/// - `5`: invalid path or invalid UTF-8
/// - `6`: unsupported special filesystem entry
/// - `7`: operation and rollback both failed; manual recovery may be required
/// - `255`: unknown error or caught panic
///
/// `preserve_ext` must be `0` or `1`.
#[no_mangle]
pub unsafe extern "C" fn exchange(
    path1: *const c_char,
    path2: *const c_char,
    preserve_ext: u8,
) -> i32 {
    ffi_boundary(|| {
        // SAFETY: The caller must uphold the pointer contracts documented above.
        let path1 = unsafe { path_from_c_string(path1) }?;
        // SAFETY: The caller must uphold the pointer contracts documented above.
        let path2 = unsafe { path_from_c_string(path2) }?;
        exchange_paths(&path1, &path2, parse_bool(preserve_ext)?)
    })
}

/// Swap names using explicit UTF-8 byte-buffer lengths.
///
/// Buffers are not required to be NUL-terminated, but embedded NUL bytes are rejected.
///
/// # Safety
///
/// For each non-zero length, the corresponding pointer must be non-null, properly aligned for
/// bytes, point to at least that many readable bytes, and remain valid and immutable for this call.
/// A null pointer is rejected even when its length is zero.
///
/// Return codes are identical to [`exchange`].
#[no_mangle]
pub unsafe extern "C" fn exchange_n(
    path1: *const u8,
    path1_len: usize,
    path2: *const u8,
    path2_len: usize,
    preserve_ext: u8,
) -> i32 {
    ffi_boundary(|| {
        // SAFETY: The caller must uphold the buffer contracts documented above.
        let path1 = unsafe { path_from_bytes(path1, path1_len) }?;
        // SAFETY: The caller must uphold the buffer contracts documented above.
        let path2 = unsafe { path_from_bytes(path2, path2_len) }?;
        exchange_paths(&path1, &path2, parse_bool(preserve_ext)?)
    })
}

/// Swap the names of two files, directories, or symbolic links.
///
/// `preserve_ext` keeps each regular file's extension while swapping stems. If it is `false`,
/// regular-file extensions are swapped too. Each underlying rename is atomic, but the complete
/// multi-step exchange is not a crash-safe filesystem transaction.
///
/// # Errors
///
/// Returns [`RenameError`] when validation or a rename/rollback operation fails.
pub fn exchange_rs(path1: &Path, path2: &Path, preserve_ext: bool) -> Result<(), RenameError> {
    exchange_paths(path1, path2, preserve_ext)
}

/// Resolve a path without dereferencing its final symbolic-link component.
///
/// # Errors
///
/// Returns [`RenameError`] when path expansion, normalization, or metadata access fails.
pub fn resolve_path_rs(path: &Path, base_dir: &Path) -> Result<(bool, PathBuf), RenameError> {
    resolve_path(path, base_dir)
}

fn ffi_boundary(operation: impl FnOnce() -> Result<(), RenameError>) -> i32 {
    match catch_unwind(AssertUnwindSafe(operation)) {
        Ok(Ok(())) => 0,
        Ok(Err(error)) => error.to_code(),
        Err(_) => 255,
    }
}

unsafe fn path_from_c_string(pointer: *const c_char) -> Result<PathBuf, RenameError> {
    if pointer.is_null() {
        return Err(RenameError::InvalidPath("null path pointer".to_owned()));
    }
    // SAFETY: The function's caller guarantees a valid NUL-terminated C string.
    let value = unsafe { CStr::from_ptr(pointer) };
    let value = value
        .to_str()
        .map_err(|error| RenameError::InvalidPath(format!("path is not UTF-8: {error}")))?;
    path_from_utf8(value)
}

unsafe fn path_from_bytes(pointer: *const u8, length: usize) -> Result<PathBuf, RenameError> {
    if pointer.is_null() {
        return Err(RenameError::InvalidPath("null path pointer".to_owned()));
    }
    // SAFETY: The function's caller guarantees that this buffer is readable for `length` bytes.
    let bytes = unsafe { slice::from_raw_parts(pointer, length) };
    if bytes.contains(&0) {
        return Err(RenameError::InvalidPath(
            "path contains an embedded NUL byte".to_owned(),
        ));
    }
    let value = str::from_utf8(bytes)
        .map_err(|error| RenameError::InvalidPath(format!("path is not UTF-8: {error}")))?;
    path_from_utf8(value)
}

fn parse_bool(value: u8) -> Result<bool, RenameError> {
    match value {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(RenameError::InvalidPath(
            "preserve_ext must be 0 or 1".to_owned(),
        )),
    }
}

fn path_from_utf8(value: &str) -> Result<PathBuf, RenameError> {
    if value.is_empty() {
        Err(RenameError::InvalidPath("path is empty".to_owned()))
    } else {
        Ok(PathBuf::from(value))
    }
}

#[cfg(test)]
mod tests {
    use std::{
        ffi::CString,
        fs,
        path::{Path, PathBuf},
    };

    use tempfile::TempDir;

    fn write(path: &Path, value: &str) {
        fs::write(path, value).expect("write test file");
    }

    fn read(path: &Path) -> String {
        fs::read_to_string(path).expect("read test file")
    }

    fn paths(dir: &TempDir, first: &str, second: &str) -> (PathBuf, PathBuf) {
        (dir.path().join(first), dir.path().join(second))
    }

    #[test]
    fn swaps_full_file_names() {
        let dir = TempDir::new().expect("create temp dir");
        let (first, second) = paths(&dir, "alpha.ext1", "beta.ext2");
        write(&first, "A");
        write(&second, "B");

        super::exchange_rs(&first, &second, false).expect("exchange files");

        assert_eq!(read(&first), "B");
        assert_eq!(read(&second), "A");
    }

    #[test]
    fn preserves_file_extensions() {
        let dir = TempDir::new().expect("create temp dir");
        let (first, second) = paths(&dir, "alpha.ext1", "beta.ext2");
        write(&first, "A");
        write(&second, "B");

        super::exchange_rs(&first, &second, true).expect("exchange files");

        assert_eq!(read(&dir.path().join("beta.ext1")), "A");
        assert_eq!(read(&dir.path().join("alpha.ext2")), "B");
    }

    #[test]
    fn preserves_spaces_and_quotes_in_names() {
        let dir = TempDir::new().expect("create temp dir");
        let (first, second) = paths(&dir, "  'alpha'.txt", "beta name.log");
        write(&first, "A");
        write(&second, "B");

        super::exchange_rs(&first, &second, false).expect("exchange files");

        assert_eq!(read(&first), "B");
        assert_eq!(read(&second), "A");
    }

    #[test]
    fn file_directory_exchange_uses_complete_names() {
        let dir = TempDir::new().expect("create temp dir");
        let file = dir.path().join("alpha.txt");
        let directory = dir.path().join("beta.dir");
        write(&file, "A");
        fs::create_dir(&directory).expect("create directory");
        write(&directory.join("inside"), "B");

        super::exchange_rs(&file, &directory, true).expect("exchange entries");

        assert_eq!(read(&directory), "A");
        assert_eq!(read(&file.join("inside")), "B");
    }

    #[test]
    fn swaps_directory_names_without_treating_dots_as_extensions() {
        let dir = TempDir::new().expect("create temp dir");
        let (first, second) = paths(&dir, "alpha.dir", "beta.folder");
        fs::create_dir(&first).expect("create first dir");
        fs::create_dir(&second).expect("create second dir");
        write(&first.join("first"), "A");
        write(&second.join("second"), "B");

        super::exchange_rs(&first, &second, true).expect("exchange dirs");

        assert_eq!(read(&first.join("second")), "B");
        assert_eq!(read(&second.join("first")), "A");
    }

    #[test]
    fn rejects_same_file() {
        let dir = TempDir::new().expect("create temp dir");
        let file = dir.path().join("same.ext");
        write(&file, "X");

        assert_eq!(
            super::exchange_rs(&file, &file, true),
            Err(super::RenameError::SamePath)
        );
    }

    #[test]
    fn rejects_nested_directories_without_mutating_them() {
        let dir = TempDir::new().expect("create temp dir");
        let parent = dir.path().join("parent");
        let child = parent.join("child");
        fs::create_dir_all(&child).expect("create nested dirs");

        assert!(matches!(
            super::exchange_rs(&parent, &child, false),
            Err(super::RenameError::InvalidPath(_))
        ));
        assert!(child.is_dir());
    }

    #[test]
    fn ffi_rejects_null_and_invalid_utf8() {
        // SAFETY: Null pointers are intentionally passed to verify validation before dereference.
        assert_eq!(
            unsafe { super::exchange(std::ptr::null(), std::ptr::null(), 0) },
            5
        );
        let invalid = [0xff_u8];
        // SAFETY: Both pointers refer to readable one-byte buffers for the call duration.
        assert_eq!(
            unsafe { super::exchange_n(invalid.as_ptr(), 1, invalid.as_ptr(), 1, 0) },
            5
        );
    }

    #[test]
    fn ffi_rejects_invalid_boolean() {
        let value = b"unused";
        // SAFETY: Both buffers are readable for the supplied length.
        assert_eq!(
            unsafe {
                super::exchange_n(value.as_ptr(), value.len(), value.as_ptr(), value.len(), 2)
            },
            5
        );
    }

    #[test]
    fn legacy_ffi_exchanges_files() {
        let dir = TempDir::new().expect("create temp dir");
        let (first, second) = paths(&dir, "one.txt", "two.txt");
        write(&first, "1");
        write(&second, "2");
        let first = CString::new(first.to_string_lossy().as_bytes()).expect("CString");
        let second = CString::new(second.to_string_lossy().as_bytes()).expect("CString");

        // SAFETY: CString pointers are valid and NUL-terminated for the call duration.
        assert_eq!(
            unsafe { super::exchange(first.as_ptr(), second.as_ptr(), 0) },
            0
        );
    }
}
