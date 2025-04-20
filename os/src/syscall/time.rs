//! time related syscall

use core::time::Duration;

use alloc::{boxed::Box, fmt, sync::Arc};
use hal::instruction::{Instruction, InstructionHal};
use log::info;

use crate::{
    processor::context::SumGuard, task::current_task, timer::{clock::{CLOCK_DEVIATION, CLOCK_MONOTONIC, CLOCK_PROCESS_CPUTIME_ID, CLOCK_REALTIME, CLOCK_THREAD_CPUTIME_ID}, ffi::{TimeSpec, TimeVal}, get_current_time_duration, get_current_time_ms, timed_task::{ksleep,suspend_timeout}, timer::{alloc_timer_id, ITimerVal, RealITimer, Timer, TIMER_MANAGER}}, utils::Select2Futures
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
    log::debug!("[sys_clock_gettime]: clock id {}", clock_id);
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
        5 => {
            log::warn!("[sys_clock_gettime] unsupported clockid{}", clock_id);
            return Err(SysError::EINTR);
        }
        _ => {
            panic!("unsupported clock id {}", clock_id);
        }
    }
    Ok(0)
}

/// Interval timer allows processes to receive signals after a specified time interval
/// set a itimer, now only irealtimer implemented
pub fn sys_setitimer(
    which: usize,
    new_ptr: usize,
    old_ptr: usize
)-> SysResult {
    if which > 2 {
        return Err(SysError::EINVAL);
    }
    let task = current_task().unwrap();
    let new =  unsafe {
        Instruction::set_sum();
        core::ptr::read(new_ptr as *const ITimerVal)
    };
    if !new.is_valid() {
        return Err(SysError::EINVAL);
    }
    let id = alloc_timer_id();
    let (prev_timeval, next_expire) = task.with_mut_itimers(|itimers|{
        let itimer = &mut itimers[which];
        let prev_timeval = ITimerVal {
            it_interval: itimer.interval.into(),
            it_value: itimer.next_expire.saturating_sub(get_current_time_duration()).into()
        };
        itimer.interval = new.it_interval.into();
        itimer.id = id;
        if new.it_value.is_zero() {
            itimer.next_expire = Duration::ZERO;
            (prev_timeval, Duration::ZERO)
        }else {
            let next_expire = get_current_time_duration() + new.it_value.into();
            itimer.next_expire = next_expire;
            (prev_timeval, next_expire)
        }
    });

    if !new.it_value.is_zero(){
        let timer = Timer::new(next_expire, Box::new(RealITimer{
            task: Arc::downgrade(&task),
            id: id
        }));
        TIMER_MANAGER.add_timer(timer);
    }
    if old_ptr != 0{
        unsafe {
            let oldptr = old_ptr as *mut ITimerVal;
            oldptr.write(prev_timeval);
        }
    }
    Ok(0)
}
/// write current itimerval into now_ptr
pub fn sys_getitimer(which: usize, now_ptr: usize) -> SysResult {
    if which > 2 {
        return Err(SysError::EINVAL);
    }
    let current = current_task().unwrap();
    if now_ptr != 0 {
        let itimerval = current.with_itimers(|itimers|{
            let itimer = &itimers[which];
            ITimerVal {
                it_interval: itimer.interval.into(),
                it_value: itimer.next_expire
                .saturating_sub(get_current_time_duration())
                .into()
            }
        });
        unsafe {
            let nowptr = now_ptr as *mut ITimerVal;
            nowptr.write(itimerval);
        }
    }
    Ok(0)
}