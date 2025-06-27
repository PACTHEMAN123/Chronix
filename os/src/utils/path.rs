//! useful utils for handling path

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use log::{info, warn};

use crate::mm::UserPtrRaw;
use crate::syscall::SysError;
use crate::utils::string;

use super::c_str_to_string;

/// translate a user space string to path
pub fn user_path_to_string(cpath: UserPtrRaw<u8>, vm: &mut crate::mm::vm::UserVmSpace ) -> Result<String, SysError> {
    if cpath.is_null() {
        return Err(SysError::EINVAL);
    }
    let slice = cpath.cstr_slice(vm)?;
    let path = slice.to_str().map_err(|_| SysError::EINVAL)?;
    if path.eq("") {
        Err(SysError::EINVAL)
    } else {
        Ok(path.to_string())
    }
}

/// get the file name using the absolute path
/// for example: a/b/c -> c | /a/b/c/ -> c
pub fn abs_path_to_name(path: &str) -> Option<String> {
    if path.is_empty() {
        return None;
    } else {
        let path = path.trim_end_matches('/');
        path.rsplit('/').next().map(|s| s.to_string())
    }
}

/// get the parent path using absolute path
/// for example: /a/b/c -> /a/b
pub fn abs_path_to_parent(path: &str) -> Option<String> {
    info!("path: {}", path);
    let path = path.trim_end_matches('/'); // 移除末尾的 `/`
    if let Some(pos) = path.rfind('/') {
        if pos == 0 {
            Some("/".to_string()) // at the root
        } else {
            Some(path[..pos].to_string()) // get the parent path
        }
    } else {
        None // invalid path: not absolute path
    }
}

/// use the parent path and the relative path
/// to generate the absolute path
/// parent path should be like: /a/b or /a/b/
/// rel path should be like: c ./c
pub fn rel_path_to_abs(parent_path: &str, rel_path: &str) -> Option<String> {
    if !parent_path.starts_with('/') {
        log::error!("parent path should be absolute path!");
        return None;
    }
    // parent path
    let mut abs_path = String::new();
    if parent_path.len() == 1 {
        // special case: '/'
        abs_path.push_str(parent_path);
    } else if parent_path.ends_with('/') {
        abs_path.push_str(parent_path);
    } else {
        abs_path.push_str(parent_path);
        abs_path.push('/');
    }
    // child path
    let rel_path = rel_path.trim_start_matches("./");
    if rel_path.is_empty() || rel_path == "." {
        return Some(abs_path.trim_end_matches('/').to_string());
    }
    abs_path.push_str(rel_path);
    Some(abs_path)
}