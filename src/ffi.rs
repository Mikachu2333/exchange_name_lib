use std::{
    ffi::{c_char, CStr},
    panic::{catch_unwind, AssertUnwindSafe},
    path::PathBuf,
    slice, str,
};

use crate::{exchange_rs, RenameError};

/// Exchanges names using NUL-terminated UTF-8 strings.
///
/// # Safety
///
/// Both pointers must address readable NUL-terminated strings that remain immutable and valid for
/// this call. `preserve_ext` must be `0` or `1`.
#[no_mangle]
pub unsafe extern "C" fn exchange(
    path1: *const c_char,
    path2: *const c_char,
    preserve_ext: u8,
) -> i32 {
    ffi_boundary(|| {
        // SAFETY: Required by this function's contract.
        let path1 = unsafe { path_from_c_string(path1) }?;
        // SAFETY: Required by this function's contract.
        let path2 = unsafe { path_from_c_string(path2) }?;
        exchange_rs(&path1, &path2, parse_bool(preserve_ext)?)
    })
}

/// Exchanges names using explicit UTF-8 buffer lengths.
///
/// # Safety
///
/// Both pointers must be non-null and readable for their supplied lengths, and the buffers must
/// remain immutable and valid for this call. `preserve_ext` must be `0` or `1`.
#[no_mangle]
pub unsafe extern "C" fn exchange_n(
    path1: *const u8,
    path1_len: usize,
    path2: *const u8,
    path2_len: usize,
    preserve_ext: u8,
) -> i32 {
    ffi_boundary(|| {
        // SAFETY: Required by this function's contract.
        let path1 = unsafe { path_from_bytes(path1, path1_len) }?;
        // SAFETY: Required by this function's contract.
        let path2 = unsafe { path_from_bytes(path2, path2_len) }?;
        exchange_rs(&path1, &path2, parse_bool(preserve_ext)?)
    })
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
        return Err(invalid("null path pointer"));
    }
    // SAFETY: Required by the caller of this function.
    let value = unsafe { CStr::from_ptr(pointer) };
    let value = value
        .to_str()
        .map_err(|error| invalid(format!("path is not UTF-8: {error}")))?;
    path_from_utf8(value)
}

unsafe fn path_from_bytes(pointer: *const u8, length: usize) -> Result<PathBuf, RenameError> {
    if pointer.is_null() {
        return Err(invalid("null path pointer"));
    }
    // SAFETY: Required by the caller of this function.
    let bytes = unsafe { slice::from_raw_parts(pointer, length) };
    if bytes.contains(&0) {
        return Err(invalid("path contains an embedded NUL byte"));
    }
    let value =
        str::from_utf8(bytes).map_err(|error| invalid(format!("path is not UTF-8: {error}")))?;
    path_from_utf8(value)
}

fn parse_bool(value: u8) -> Result<bool, RenameError> {
    match value {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(invalid("preserve_ext must be 0 or 1")),
    }
}

fn path_from_utf8(value: &str) -> Result<PathBuf, RenameError> {
    if value.is_empty() {
        Err(invalid("path is empty"))
    } else {
        Ok(PathBuf::from(value))
    }
}

fn invalid(message: impl Into<String>) -> RenameError {
    RenameError::InvalidPath(message.into())
}
