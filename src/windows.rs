use std::{
    ffi::OsString,
    io,
    os::windows::ffi::{OsStrExt, OsStringExt},
    path::Path,
};

use windows_sys::Win32::{
    Foundation::INVALID_HANDLE_VALUE,
    Storage::FileSystem::{FindClose, FindFirstFileW, WIN32_FIND_DATAW},
};

pub(crate) fn entry_name(path: &Path) -> io::Result<OsString> {
    let invalid = || io::Error::new(io::ErrorKind::InvalidInput, "expected a literal entry name");
    let name = path.file_name().ok_or_else(invalid)?;
    if name.encode_wide().any(|unit| matches!(unit, 42 | 63)) {
        return Err(invalid());
    }
    let mut wide: Vec<_> = path.as_os_str().encode_wide().collect();
    if wide.contains(&0) {
        return Err(invalid());
    }
    wide.push(0);
    let mut data = WIN32_FIND_DATAW::default();
    let handle = unsafe { FindFirstFileW(wide.as_ptr(), &raw mut data) };
    if handle == INVALID_HANDLE_VALUE {
        return Err(io::Error::last_os_error());
    }
    unsafe { FindClose(handle) };
    let end = data
        .cFileName
        .iter()
        .position(|&unit| unit == 0)
        .unwrap_or(data.cFileName.len());
    Ok(OsString::from_wide(&data.cFileName[..end]))
}
