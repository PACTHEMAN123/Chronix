//! time related syscall

use core::time::Duration;

use log::info;

use crate::{
    processor::context::SumGuard, task::current_task, timer::{ffi::{TimeSpec, TimeVal}, get_current_time_ms,timed_task::{ksleep,suspend_timeout}}, utils::Select2Futures
};
use super::{SysError, SysResult};
/// get current time of day
pub fn sys_gettimeofday(tv: *mut TimeVal) -> SysResult {
    let _sum_guard = SumGuard::new();
    let current_time = get_current_time_ms();
    let time_val = TimeVal {
        sec: current_time / 1000,
        usec: (current_time % 1000) * 1000,
    };
    
    unsafe {
        tv.write_volatile(time_val);
    }
    Ok(0)
}
use crate::timer::ffi::Tms;
/// times syscall
pub fn sys_times(tms: *mut Tms) -> SysResult {
    let _sum_guard = SumGuard::new();
    let current_task = current_task().unwrap();
    let tms_val = Tms::from_time_recorder(current_task.time_recorder());
    unsafe {
        tms.write_volatile(tms_val);
    }
    Ok(0)
}
/// sleep syscall
pub async fn sys_nanosleep(time_ptr: usize, time_out_ptr: usize) -> SysResult {
    let time_val_ptr = time_ptr as *const TimeSpec;
    let time_val = unsafe {*time_val_ptr};
    let sleep_time_duration = time_val.into();
    let remain = suspend_timeout(current_task().unwrap(), sleep_time_duration).await;
    if remain.is_zero(){
        Ok(0)
    }else{
        unsafe {
            (time_out_ptr as *mut TimeSpec).write(remain.into());
        }
        Err(SysError::EINTR)
    }
}