//! time related syscall

use core::{fmt::Error, ops::DerefMut, time::Duration};

use alloc::{boxed::Box, fmt, sync::Arc, task};
use fatfs::{info, Time};
use hal::instruction::{Instruction, InstructionHal};
use xmas_elf::program::Flags;

use crate::{mm::UserPtrRaw, processor::context::SumGuard, task::current_task, timer::{clock::{CLOCK_DEVIATION, CLOCK_MONOTONIC, CLOCK_MONOTONIC_COARSE, CLOCK_MONOTONIC_RAW, CLOCK_PROCESS_CPUTIME_ID, CLOCK_REALTIME, CLOCK_REALTIME_COARSE, CLOCK_THREAD_CPUTIME_ID}, ffi::{TimeSpec, TimeVal}, get_current_time_duration, get_current_time_ms, get_current_time_us, timed_task::{ksleep,suspend_timeout}, timer::{alloc_timer_id, ITimerVal, RealITimer, Timer, TIMER_MANAGER}}, utils::Select2Futures
};
use super::{SysError, SysResult};
/// get current time of day
pub fn sys_gettimeofday(tv: usize) -> SysResult {
    let task = current_task().unwrap();
    let mut vm = task.get_vm_space().lock();
    if tv != 0 {
        let tv_ptr = UserPtrRaw::new(tv as *mut TimeVal)
        .ensure_write(&mut vm)
        .ok_or(SysError::EFAULT)?;
        let current_time = get_current_time_us();
        let time_val = TimeVal {
            sec: current_time / 1_000_000,
            usec: (current_time % 1_000_000),
        };
        tv_ptr.write(time_val);
    }
    Ok(0)
}
use crate::timer::ffi::Tms;
/// times syscall
pub fn sys_times(tms: usize) -> SysResult {
    let task = current_task().unwrap();
    if tms != 0 {
        let tms_ptr = UserPtrRaw::new(tms as *mut Tms)
            .ensure_write(&mut task.get_vm_space().lock())
            .ok_or(SysError::EFAULT)?;
        let current_task = current_task().unwrap();
        let tms_val = Tms::from_time_recorder(current_task.time_recorder());
        tms_ptr.write(tms_val);
    }
    Ok(0)
}
/// sleep syscall
pub async fn sys_nanosleep(time_ptr: usize, time_out_ptr: usize) -> SysResult {
    let task = current_task().unwrap();
    if time_ptr == 0 {
        return Ok(0);
    }
    let time_val_ptr = 
        UserPtrRaw::new(time_ptr as *const TimeSpec)
            .ensure_read(&mut task.get_vm_space().lock())
            .ok_or(SysError::EFAULT)?;
    let time_val = *time_val_ptr.to_ref(); 
    if !time_val.is_valid() {
        return  Err(SysError::EINVAL);
    }
    let sleep_time_duration = time_val.into();
    let remain = suspend_timeout(current_task().unwrap(), sleep_time_duration).await;
    if remain.is_zero() {
        Ok(0)
    } else {
        // *time_out = remain.into();
        // Err(SysError::EINTR)
        if time_out_ptr != 0 {
            let time_out_ptr = 
            UserPtrRaw::new(time_out_ptr as *const TimeSpec)
                .ensure_write(&mut task.get_vm_space().lock())
                .ok_or(SysError::EFAULT)?;
            time_out_ptr.write(remain.into());
        }
        Err(SysError::EINTR)
    }
    
}

/// syscall: clock_gettime
pub fn sys_clock_gettime(clock_id: usize, ts: usize) -> SysResult {
    let task = current_task().unwrap().clone();
    if ts == 0{
        return Ok(0);
    }
    let ts_ptr = UserPtrRaw::new(ts as *mut TimeSpec)
        .ensure_write(&mut task.get_vm_space().lock())
        .ok_or(SysError::EFAULT)?;
    // log::info!("[sys_clock_gettime]: clock id {}", clock_id);
    match clock_id {
        CLOCK_REALTIME | CLOCK_MONOTONIC => {
            let current = get_current_time_duration();
            unsafe {
                ts_ptr.write((CLOCK_DEVIATION[clock_id] + current).into());
            }
        }
        CLOCK_MONOTONIC_RAW => {
            let current = get_current_time_duration();
            unsafe {
                ts_ptr.write((CLOCK_DEVIATION[CLOCK_MONOTONIC] + current).into());
            }
        }
        CLOCK_PROCESS_CPUTIME_ID => {
            let cpu_time = task.process_cpu_time();
            ts_ptr.write(cpu_time.into()); 
        }
        CLOCK_THREAD_CPUTIME_ID => {
            let (user_time, kernel_time) = task.time_recorder().time_pair();
            let cpu_time = user_time + kernel_time;
            ts_ptr.write(cpu_time.into());
        }
        CLOCK_REALTIME_COARSE => {
            let current = get_current_time_duration();
            unsafe {
                ts_ptr.write((CLOCK_DEVIATION[CLOCK_REALTIME] + current).into());
            }
        }
        CLOCK_MONOTONIC_COARSE => {
            let current = get_current_time_duration();
            unsafe {
                ts_ptr.write((CLOCK_DEVIATION[CLOCK_MONOTONIC] + current).into());
            }
        }
        _ => {
            log::warn!("[sys_clock_gettime] unsupported clockid {}", clock_id);
            return Err(SysError::EINVAL);
        }
    }
    Ok(0)
}

pub fn sys_clock_settime(clock_id: usize, ts_ptr: usize) -> SysResult {
    if clock_id == CLOCK_PROCESS_CPUTIME_ID
            || clock_id == CLOCK_THREAD_CPUTIME_ID
            || clock_id == CLOCK_MONOTONIC
    {
        return Err(SysError::EINVAL);
    }
    let task = current_task().unwrap().clone();
    let tp = *UserPtrRaw::new(ts_ptr as *const TimeSpec)
        .ensure_read(&mut task.vm_space.lock())
        .ok_or(SysError::EFAULT)?
        .to_ref();
    let duration: Duration = tp.into();
    if !tp.is_valid() {
        return Err(SysError::EINVAL);
    }
    match clock_id {
        CLOCK_REALTIME => {
            if tp.into_ms() < get_current_time_ms() {
                return Err(SysError::EINVAL);
            }
            unsafe {
                CLOCK_DEVIATION[clock_id] = duration - get_current_time_duration();
            }
        }
        _ => {
            log::warn!("[clock_settime] unsupport clock {clock_id}");
            return Err(SysError::EINVAL);
        }
    }
    Ok(0)
}

/// syscall: sys clock getres
/// clock_getres() finds the resolution (precision) of
/// the specified clock clockid, and, if res is non-NULL, stores it in
/// the struct timespec pointed to by res.  The resolution of clocks
/// depends on the implementation and cannot be configured by a
/// particular process.
pub fn sys_clock_getres(_clockid: usize, res_ptr: usize) -> SysResult {
    if res_ptr == 0 {
        return Ok(0)
    }
    let task = current_task().unwrap().clone();
    let res_ptr = UserPtrRaw::new(res_ptr as *const TimeSpec)
        .ensure_write(&mut task.get_vm_space().lock())
        .ok_or(SysError::EFAULT)?;
    let res = res_ptr.to_mut();
    *res = Duration::from_nanos(1).into();
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
    let new =  *(UserPtrRaw::new(new_ptr as *const ITimerVal)
        .ensure_read(&mut task.get_vm_space().lock())
        .ok_or(SysError::EFAULT)?
        .to_ref());
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
            task: Arc::downgrade(task),
            id: id
        }));
        TIMER_MANAGER.add_timer(timer);
    }
    // if old_ptr != 0{
    //     unsafe {
    //         let oldptr = old_ptr as *mut ITimerVal;
    //         oldptr.write(prev_timeval);
    //     }
    // }
    if old_ptr != 0 {
        let old_ptr = UserPtrRaw::new(old_ptr as *mut ITimerVal)
            .ensure_write(&mut task.get_vm_space().lock())
            .ok_or(SysError::EFAULT)?;
        old_ptr.write(prev_timeval);
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
        let now_ptr   = UserPtrRaw::new(now_ptr as *mut ITimerVal)
            .ensure_write(&mut current.get_vm_space().lock())
            .ok_or(SysError::EFAULT)?;
        now_ptr.write(itimerval);
    }
    Ok(0)
}

/// clock_nanosleep is a more general version of nanosleep, 
/// which allows for more precise timing control.
pub async fn sys_clock_nanosleep(
    clock_id: usize,
    flags: usize,
    t_ptr: usize,
    rem_ptr: usize
) -> SysResult {
    let task = current_task().unwrap();
    match clock_id {
        CLOCK_REALTIME | CLOCK_MONOTONIC => {
            let t = *(UserPtrRaw::new(t_ptr as *const TimeSpec)
                .ensure_read(&mut task.get_vm_space().lock())
                .ok_or(SysError::EFAULT)?
                .to_ref());
            if !t.is_valid() {
                return  Err(SysError::EINVAL);
            }
            let req_time: Duration = t.into();
            let remain_time = if flags == 1 {
                let current_time = get_current_time_duration();
                if req_time.le(&current_time){
                    return Ok(0);
                }
                let sleep_time = req_time - current_time;
                suspend_timeout(task, sleep_time).await
            }else {
                suspend_timeout(task, req_time).await
            };
            if remain_time.is_zero() {
                Ok(0)
            }else {
                log::warn!("[sys_clock_nanosleep] rem_ptr: {}", rem_ptr);
                let remptr = UserPtrRaw::new(rem_ptr as *mut TimeSpec)
                    .ensure_write(&mut task.get_vm_space().lock())
                    .ok_or(SysError::EFAULT)?;
                if rem_ptr != 0 {
                    remptr.write(remain_time.into());
                }
                Err(SysError::EINTR)
            }
        }
        _ => {
            return Err(SysError::EOPNOTSUPP);
        }
    }
}

/// from linux
#[derive(Clone, Copy, Debug,Default)]
#[repr(C)]
pub struct KernTimex {
    // unsigned int
    pub kt_modes: u32, 
    // padding
    _pad0: u32,     
    // long long
    pub kt_offset: i64, 
    pub kt_freq: i64,
    pub kt_maxerror: i64,
    pub kt_esterror: i64,
    // int
    pub kt_status: i32,
    // padding 
    _pad1: u32,      

    pub kt_constant: i64,
    pub kt_precision: i64,
    pub kt_tolerance: i64,

    pub kt_time: TimeVal, 

    pub kt_tick: i64,
    pub kt_ppsfreq: i64,
    pub kt_jitter: i64,
    pub kt_shift: i32,
    _pad2: u32,

    pub kt_stabil: i64,
    pub kt_jitcnt: i64,
    pub kt_calcnt: i64,
    pub kt_errcnt: i64,
    pub kt_stbcnt: i64,

    pub kt_tai: i32,

    // padding data from linux
    _pad_last: [u32; 11],
}

bitflags! {
    pub struct TimexModes: u32 {
        const ADJ_OFFSET           = 0x0001;
        const ADJ_FREQUENCY        = 0x0002;
        const ADJ_MAXERROR         = 0x0004;
        const ADJ_ESTERROR         = 0x0008;
        const ADJ_STATUS           = 0x0010;
        const ADJ_TIMECONST        = 0x0020;
        const ADJ_TAI              = 0x0080;
        const ADJ_SETOFFSET        = 0x0100;
        const ADJ_MICRO            = 0x1000;
        const ADJ_NANO             = 0x2000;
        const ADJ_TICK             = 0x4000;
        // Userland only
        const ADJ_OFFSET_SINGLESHOT = 0x8001;
        const ADJ_OFFSET_SS_READ    = 0xa001;
    }
}

bitflags! {
    /// Status bits for struct timex.status
    pub struct TimexStatus: u32 {
        const STA_PLL        = 0x0001;
        const STA_PPSFREQ    = 0x0002;
        const STA_PPSTIME    = 0x0004;
        const STA_FLL        = 0x0008;
        const STA_INS        = 0x0010;
        const STA_DEL        = 0x0020;
        const STA_UNSYNC     = 0x0040;
        const STA_FREQHOLD   = 0x0080;
        const STA_PPSSIGNAL  = 0x0100;
        const STA_PPSJITTER  = 0x0200;
        const STA_PPSWANDER  = 0x0400;
        const STA_PPSERROR   = 0x0800;
        const STA_CLOCKERR   = 0x1000;
        const STA_NANO       = 0x2000;
        const STA_MODE       = 0x4000;
        const STA_CLK        = 0x8000;

        /// Read-only bits mask
        const STA_RONLY = Self::STA_PPSSIGNAL.bits()
                         | Self::STA_PPSJITTER.bits()
                         | Self::STA_PPSWANDER.bits()
                         | Self::STA_PPSERROR.bits()
                         | Self::STA_CLOCKERR.bits()
                         | Self::STA_NANO.bits()
                         | Self::STA_MODE.bits()
                         | Self::STA_CLK.bits();
    }
}
// global variable save last none 0 set kerntimex
pub static mut TIMEX: KernTimex = KernTimex {
    kt_modes:     0,
    _pad0:     0,
    kt_offset:    0,
    kt_freq:      0,
    kt_maxerror:  0,
    kt_esterror:  0,
    kt_status:    0,
    _pad1:     0,
    kt_constant:  0,
    kt_precision: 0,
    kt_tolerance: 0,
    kt_time:      TimeVal { sec: 0, usec: 0 },
    kt_tick:      10000,
    kt_ppsfreq:   0,
    kt_jitter:    0,
    kt_shift:     0,
    _pad2:     0,
    kt_stabil:    0,
    kt_jitcnt:    0,
    kt_calcnt:    0,
    kt_errcnt:    0,
    kt_stbcnt:    0,
    kt_tai:       0,
    _pad_last:[0; 11],
};


/// kernel time adjustment
pub fn sys_adjtimex(timex: usize) -> SysResult {
    /// do_adjtimex in linux
    fn do_adjtimex(timex: &mut KernTimex) -> SysResult {
        if timex.kt_modes == 0x80000 {
            return Err(SysError::EINVAL);
        }
        let support_mode = TimexModes::all();
        let modes = match TimexModes::from_bits(timex.kt_modes) {
            None => return Err(SysError::EINVAL),
            Some(mode) => {
                if !(mode & !support_mode).is_empty() {
                    return Err(SysError::EINVAL);
                }else {
                    mode
                }
            }
        };
        let mut ret = 0;
        if modes.contains(TimexModes::ADJ_SETOFFSET) {
            let mut delta = TimeSpec::ZERO;
            delta.tv_sec = timex.kt_time.sec;
            delta.tv_nsec = timex.kt_time.usec;
            if modes.contains(TimexModes::ADJ_NANO) {
                delta.tv_nsec *= 1000;
            }
            ret = add_offset(&delta)?;
        }
        if modes.contains(TimexModes::ADJ_TICK) {
            let tick = timex.kt_tick;
            if tick < 9000 || tick > 11000 {
                return Err(SysError::EINVAL);
            }
        }
        Ok(ret)
    }

    /// helper func for delta in kern clock
    fn add_offset(delta: &TimeSpec) -> SysResult {
        if !delta.is_valid(){
            return Err(SysError::EINVAL);
        }
        let wall_time = TimeSpec::wall_time();
        let new_time = TimeSpec {
            tv_sec: wall_time.tv_sec + delta.tv_sec,
            tv_nsec: wall_time.tv_nsec + delta.tv_nsec,
        };
        let delta_dur: Duration = (*delta).into();
        let wall_time_dur: Duration = wall_time.into();
        if delta_dur > (get_current_time_duration() - wall_time_dur) || !new_time.is_valid() {
            return Err(SysError::EINVAL);
        }
        Ok(0)
    }
    let task = current_task().unwrap();
    let mut r_timex = *UserPtrRaw::new(timex as *mut KernTimex)
    .ensure_read(&mut task.get_vm_space().lock())
    .ok_or(SysError::EFAULT)?
    .to_ref();
    let modes = r_timex.kt_modes; 
    if modes == 0x8000 {
        return Err(SysError::EINVAL);
    }
    let w_timex = UserPtrRaw::new(timex as *mut KernTimex)
        .ensure_write(&mut task.get_vm_space().lock())
        .ok_or(SysError::EFAULT)?;
    if modes == 0 {
        unsafe {w_timex.write(TIMEX)};
        return Ok(0);   
    }

    let status = do_adjtimex(&mut r_timex)?;
    w_timex.to_mut().kt_tick = 10000;
    unsafe {TIMEX.clone_from(w_timex.to_mut());}
    Ok(status)
}

/// able to choose which clock compared tp adjtimex
pub fn sys_clock_adjtime(clock_id: usize, timex: usize) -> SysResult {
    let task = current_task().unwrap();
    let _timex = UserPtrRaw::new(timex as *mut KernTimex)
        .ensure_read(&mut task.get_vm_space().lock())
        .ok_or(SysError::EFAULT)?
        .to_ref();
    match clock_id {
        CLOCK_REALTIME  => {
            sys_adjtimex(timex)
        }
        _ => {
            Ok(0)
        }
    }
}