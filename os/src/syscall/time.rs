//! time related syscall

use core::time::Duration;

use log::info;

use crate::{
    processor::context::SumGuard, task::current_task, timer::{clock::{CLOCK_DEVIATION, CLOCK_MONOTONIC, CLOCK_PROCESS_CPUTIME_ID, CLOCK_REALTIME, CLOCK_THREAD_CPUTIME_ID}, ffi::{TimeSpec, TimeVal}, get_current_time_duration, get_current_time_ms, timed_task::{ksleep,suspend_timeout}}, utils::Select2Futures
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

/// syscall: clock_gettime
pub fn sys_clock_gettime(clock_id: usize, ts: usize) -> SysResult {
    let _sum_guard = SumGuard::new();
    if ts == 0 {
        return Ok(0)
    }
    let ts_ptr = ts as *mut TimeSpec;

    match clock_id {
        CLOCK_REALTIME | CLOCK_MONOTONIC => {
            let current = get_current_time_duration();
            unsafe {
                ts_ptr.write((CLOCK_DEVIATION[clock_id] + current).into());
            }
        }
        CLOCK_PROCESS_CPUTIME_ID => {
            let cpu_time = current_task().unwrap().process_cpu_time();
            unsafe { ts_ptr.write(cpu_time.into()); }
        }
        CLOCK_THREAD_CPUTIME_ID => {
            let (user_time, kernel_time) = current_task().unwrap().time_recorder().time_pair();
            let cpu_time = user_time + kernel_time;
            unsafe { ts_ptr.write(cpu_time.into()); }
        }
        _ => {
            panic!("unsupported clock id {}", clock_id);
        }
    }
    Ok(0)
}