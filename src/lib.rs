use std::ffi::{c_char, CStr};
use std::path::PathBuf;

mod exchange;
mod file_rename;
mod path_checkout;
mod types;

use crate::exchange::exchange_paths;
use crate::types::RenameError;

#[no_mangle]
/// # Safety
/// 最终暴露的执行函数，传入两个路径String，返回一个状态码
///
/// 0 => Success，1 => No Exist
///
/// 2 => Permission Denied，3 => New File Already Exists
///
/// 255 => Unknown Error
pub unsafe extern "C" fn exchange(path1: *const c_char, path2: *const c_char) -> i32 {
    unsafe { convert_inputs(path1, path2) }
        .and_then(|(path1, path2)| exchange_paths(path1, path2))
        .map(|_| 0)
        .unwrap_or_else(|err| {
            eprintln!("{}", err);
            err.to_code()
        })
}

unsafe fn convert_inputs(
    path1: *const c_char,
    path2: *const c_char,
) -> Result<(PathBuf, PathBuf), RenameError> {
    let path1 = ptr_to_path(path1)?;
    let path2 = ptr_to_path(path2)?;
    Ok((path1, path2))
}

unsafe fn ptr_to_path(ptr: *const c_char) -> Result<PathBuf, RenameError> {
    if ptr.is_null() {
        return Err(RenameError::NotExists);
    }

    let c_str = CStr::from_ptr(ptr);
    let raw = c_str.to_string_lossy();
    let sanitized = sanitize_input(raw.as_ref());

    if sanitized.is_empty() {
        return Err(RenameError::NotExists);
    }

    Ok(PathBuf::from(sanitized))
}

fn sanitize_input(input: &str) -> String {
    input
        .trim()
        .trim_matches(|c| c == '"' || c == '\'')
        .to_string()
}

#[cfg(test)]
mod tests {
    use std::{
        ffi::CString,
        path::{Path, PathBuf},
    };

    fn clear_olds() -> (PathBuf, PathBuf) {
        let file1 = Path::new(r"D:\Desktop\f\s\2.txt");
        let file2 = Path::new(r"D:\Desktop\f");
        (file1.to_path_buf(), file2.to_path_buf())
    }

    #[test]
    fn it_works() {
        let (file1, file2) = clear_olds();
        // 0 => Success，1 => No Exist
        // 2 => Permission Denied，3 => New File Already Exists
        let trans = |s: PathBuf| CString::new(s.to_str().unwrap()).unwrap();
        let test_path1 = trans(file1);
        let test_path2 = trans(file2);

        unsafe {
            super::exchange(test_path1.as_ptr(), test_path2.as_ptr());
        }
    }
}
