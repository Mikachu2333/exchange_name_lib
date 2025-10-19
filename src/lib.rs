use std::{
    ffi::{c_char, CStr},
    path::PathBuf,
};

use file_rename::NameExchange;
use path_checkout::GetPathInfo;
mod file_rename;
mod path_checkout;

#[no_mangle]
/// # Safety
/// 最终暴露的执行函数，传入两个路径String，返回一个u8
///
/// 0 => Success，1 => No Exist
///
/// 2 => Permission Denied，3 => New File Already Exists
///
/// 255 => UNKNOWN ERROR
pub extern "C" fn exchange(path1: *const c_char, path2: *const c_char) -> i32 {
    let binding = std::env::current_exe().unwrap();
    let exe_dir = binding.parent().unwrap();

    if path1.is_null() || path2.is_null() {
        return 255_i32;
    }

    let transformer = |s: *const c_char| unsafe { CStr::from_ptr(s) }.to_string_lossy().to_string();

    let raw1 = transformer(path1);
    let raw2 = transformer(path2);

    let mut all_infos = NameExchange::new();

    let strip_wrapping_quotes = |s: &str| -> &str {
        let s = s.trim();
        if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
            &s[1..s.len() - 1]
        } else {
            s
        }
    };

    let path_check = |s: String| {
        let unquoted = strip_wrapping_quotes(&s).to_string();

        let mut candidate_str = unquoted.clone();

        if cfg!(target_os = "linux") {
            let is_wsl_env = || -> bool {
                std::fs::read_to_string("/proc/version")
                    .map(|v| v.contains("Microsoft") || v.contains("WSL"))
                    .unwrap_or(false)
            };

            if is_wsl_env() {
                let bytes = unquoted.as_bytes();
                let looks_like_win_drive = bytes.len() > 2
                    && bytes[1] == b':'
                    && (bytes[2] == b'/' || bytes[2] == b'\\')
                    && ((bytes[0] >= b'A' && bytes[0] <= b'Z')
                        || (bytes[0] >= b'a' && bytes[0] <= b'z'));

                if looks_like_win_drive {
                    let drive = unquoted.chars().next().unwrap().to_ascii_lowercase();
                    let rest = unquoted[2..].replace('\\', "/");
                    let rest = rest.trim_start_matches(['/','\\']);
                    candidate_str = format!("/mnt/{}/{}", drive, rest);
                } else if unquoted.starts_with("\\\\") {
                    let without_prefix = &unquoted[2..];
                    let mut parts = without_prefix.split(['\\','/']).filter(|s| !s.is_empty());
                    if let (Some(server), Some(share)) = (parts.next(), parts.next()) {
                        let rest = parts.collect::<Vec<_>>().join("/");
                        if rest.is_empty() {
                            candidate_str = format!("/mnt/unc/{}/{}", server, share);
                        } else {
                            candidate_str = format!("/mnt/unc/{}/{}/{}", server, share, rest);
                        }
                    }
                }
            }
        }

        let p = PathBuf::from(&candidate_str);
        if p.exists() {
            p.canonicalize().unwrap_or(p)
        } else {
            p
        }
    };

    let mut packed_path = GetPathInfo {
        path1: path_check(raw1),
        path2: path_check(raw2),
    };

    (all_infos.f1.is_exist, all_infos.f2.is_exist) = (packed_path).if_exist(exe_dir);
    if (!all_infos.f1.is_exist) || (!all_infos.f2.is_exist) {
        return 1_i32;
    }
    if packed_path.path1 == packed_path.path2 {
        return 2_i32;
    }
    all_infos.f1.exchange.original_path = packed_path.path1.clone();
    all_infos.f2.exchange.original_path = packed_path.path2.clone();

    (all_infos.f1.is_file, all_infos.f2.is_file) = packed_path.if_file();

    (all_infos.f1.packed_info, all_infos.f2.packed_info) =
        packed_path.metadata_collect(all_infos.f1.is_file, all_infos.f2.is_file);

    (
        all_infos.f1.exchange.pre_path,
        all_infos.f1.exchange.new_path,
    ) = NameExchange::make_name(
        &all_infos.f1.packed_info.parent_dir,
        &all_infos.f2.packed_info.name,
        &all_infos.f1.packed_info.ext,
    );
    (
        all_infos.f2.exchange.pre_path,
        all_infos.f2.exchange.new_path,
    ) = NameExchange::make_name(
        &all_infos.f2.packed_info.parent_dir,
        &all_infos.f1.packed_info.name,
        &all_infos.f2.packed_info.ext,
    );

    let mut packed_path_new = GetPathInfo {
        path1: all_infos.f1.exchange.new_path.clone(),
        path2: all_infos.f2.exchange.new_path.clone(),
    };
    let (exist_new_1, exist_new_2) = GetPathInfo::if_exist(&mut packed_path_new, exe_dir);
    let same_dir = GetPathInfo::if_same_dir(&packed_path_new);
    if !same_dir && (exist_new_1 || exist_new_2) {
        //不能因为rename函数里面有就删了……
        /*
        println!(
            "same:{}\tnew1:{}\tnew2:{}",
            same_dir, exist_new_1, exist_new_2
        );
        */
        return 3_i32;
    }

    //1 -> parent1, 2 -> parent2
    let mode = packed_path.if_root();



    match (all_infos.f1.is_file, all_infos.f2.is_file) {
        (true, true) => NameExchange::rename_each(&all_infos, false, true),
        (false, false) => {
            // 都是目录，检查包含关系
            match mode {
                1 => NameExchange::rename_each(&all_infos, true, false),
                2 => NameExchange::rename_each(&all_infos, true, true),
                _ => NameExchange::rename_each(&all_infos, false, true),
            }
        }
        (true, false) => {
            if mode == 2 {
                NameExchange::rename_each(&all_infos, true, true)
            } else {
                NameExchange::rename_each(&all_infos, false, true)
            }
        }
        (false, true) => {
            if mode == 1 {
                NameExchange::rename_each(&all_infos, true, false)
            } else {
                NameExchange::rename_each(&all_infos, false, false)
            }
        }
    }
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
        return (file1.to_path_buf(), file2.to_path_buf());
    }

    #[test]
    fn it_works() {
        let (file1, file2) = clear_olds();
        // 0 => Success，1 => No Exist
        // 2 => Permission Denied，3 => New File Already Exists
        let trans = |s: PathBuf| CString::new(s.to_str().unwrap()).unwrap();
        let test_path1 = trans(file1);
        let test_path2 = trans(file2);

        super::exchange(test_path1.as_ptr(), test_path2.as_ptr());
    }
}
