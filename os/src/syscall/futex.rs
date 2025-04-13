use crate::timer::ffi::TimeSpec;

use super::{SysError, SysResult};

/// Robust List Head
pub struct RobustListHead;

/// get futex
#[allow(unused_variables)]
pub fn sys_futex(
    uaddr: *const u32, futex_op: i32, val: u32, 
    timeout: *const TimeSpec, uaddr2: *const u32, val3: u32
) -> SysResult {
    Err(SysError::ENOSYS)
}

/// get robust list
#[allow(unused_variables)]
pub fn sys_get_robust_list(
    pid: i32, head_ptr: *mut *const RobustListHead, len_ptr: *mut usize
) -> SysResult {
    Err(SysError::ENOSYS)
}

/// set robust list
#[allow(unused_variables)]
pub fn sys_set_robust_list(head: *const RobustListHead, len_ptr: usize) -> SysResult {
    Err(SysError::ENOSYS)
}


