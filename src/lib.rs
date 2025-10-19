use std::ffi::{c_char, CStr};
use std::path::{Path, PathBuf};

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

pub fn exchange_rs(path1: &Path, path2: &Path) -> Result<(), types::RenameError> {
    exchange_paths(path1.to_path_buf(), path2.to_path_buf())
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
        fs::{self, remove_file},
        path::PathBuf,
    };

    fn clear_olds() -> (PathBuf, PathBuf) {
        let current_exe = std::env::current_exe().unwrap();
        let base_dir = current_exe.parent().unwrap();
        let _ = std::env::set_current_dir(base_dir);

        let original_path1 = r"\\wsl.localhost\Debian\home\LinkChou\";
        let original_path2 = r"";

        let file1 = "1.ext1";
        let file2 = "2.ext2";

        let exchanged_file1 = "2.ext1";
        let exchanged_file2 = "1.ext2";

        let path1 = format!(r"{}{}", original_path1, file1);
        let path2 = format!(r"{}{}", original_path2, file2);

        let exchanged_path1 = format!(r"{}{}", original_path1, exchanged_file1);
        let exchanged_path2 = format!(r"{}{}", original_path2, exchanged_file2);

        let _ = remove_file(exchanged_path1);
        let _ = remove_file(exchanged_path2);
        let _ = fs::File::create(file1);
        let _ = fs::File::create(file2);

        (PathBuf::from(path1), PathBuf::from(path2))
    }

    #[test]
    fn it_works() {
        let (file1, file2) = clear_olds();

        match super::exchange_rs(&file1, &file2) {
            Ok(_) => {
                println!("Success");
            }
            Err(e) => {
                eprintln!("{}", e);
            }
        };
    }
}
