use std::{ffi::CString, fs};

use exchange_name_lib::{exchange, exchange_n};
use tempfile::TempDir;

#[test]
fn rejects_null_and_invalid_utf8() {
    // SAFETY: Null pointers verify validation before dereference.
    assert_eq!(
        unsafe { exchange(std::ptr::null(), std::ptr::null(), 0) },
        5
    );

    let invalid = [0xff_u8];
    // SAFETY: Both pointers address readable one-byte buffers.
    assert_eq!(
        unsafe { exchange_n(invalid.as_ptr(), 1, invalid.as_ptr(), 1, 0) },
        5
    );
}

#[test]
fn rejects_invalid_boolean() {
    let value = b"unused";
    // SAFETY: Both buffers are readable for the supplied lengths.
    assert_eq!(
        unsafe { exchange_n(value.as_ptr(), value.len(), value.as_ptr(), value.len(), 2) },
        5
    );
}

#[test]
fn legacy_interface_exchanges_files() {
    let dir = TempDir::new().expect("create temp dir");
    let first = dir.path().join("one.txt");
    let second = dir.path().join("two.txt");
    fs::write(&first, "1").expect("write first");
    fs::write(&second, "2").expect("write second");
    let first = CString::new(first.to_string_lossy().as_bytes()).expect("CString");
    let second = CString::new(second.to_string_lossy().as_bytes()).expect("CString");

    // SAFETY: CString pointers are valid and NUL-terminated for this call.
    assert_eq!(unsafe { exchange(first.as_ptr(), second.as_ptr(), 0) }, 0);
}
