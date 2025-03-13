//! useful utils for handling path

use alloc::string::{String, ToString};

use crate::mm::UserCheck;
use crate::utils::string;

use super::c_str_to_string;

/// translate a user space string to path
pub fn user_path_to_string(cpath: *const u8) -> Option<String> {
    if cpath.is_null() {
        return None;
    }
    let path = c_str_to_string(cpath);
    if path.eq("") {
        None
    } else {
        Some(path)
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