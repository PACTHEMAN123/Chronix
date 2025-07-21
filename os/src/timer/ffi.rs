use alloc::collections::btree_map::Values;
use crate::timer::get_current_time_ns;

use super::{get_current_time_ms, NSEC_PER_SEC};
use core::time::Duration;

use super::{USEC_PER_SEC,MSEC_PER_SEC};

#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
/// TimeVal struct for syscall, TimeVal stans for low-precision time value
pub struct TimeVal {
    /// seconds
    pub sec: usize,
    /// microseconds
    pub usec: usize,
}


#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
/// TimeSpec struct, TimeSpec stands for high-precision time value
pub struct TimeSpec {
    /// sec
    pub tv_sec: usize,
    /// nano sec
    pub tv_nsec: usize,
}

impl From<Duration> for TimeVal{
    fn from(value: Duration) -> Self {
        Self { sec: value.as_secs() as usize, usec: value.subsec_micros() as usize }
    }
}

impl Into<Duration> for TimeVal{
    fn into(self) -> Duration {
        Duration::new(self.sec as u64, (self.usec * MSEC_PER_SEC) as u32)
    }
}

impl TimeVal {
    /// Const ZERO for TimeVal
    pub const ZERO: Self = Self { sec: 0, usec: 0 };
    /// new TimeVal from a single value in microseconds
    pub fn from_usec(usec: usize) -> Self{
        Self {
            sec: usec / USEC_PER_SEC,
            usec: usec % USEC_PER_SEC,
        }
    }
    /// calculate the microseconds of TimeVal
    pub fn into_usec(&self) -> usize {
        self.sec * USEC_PER_SEC + self.usec
    } 
    /// check if is zero
    pub fn is_zero(&self) -> bool {
        self.sec == 0 && self.usec == 0
    }
    /// check if is valid
    pub fn is_valid(&self) -> bool {
        self.sec as isize > 0 && self.usec as isize >= 0 && self.usec  < 1000_000 
    }

    /// to bytes
    pub fn _as_bytes(time: &Self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts((time as *const Self) as *const u8, core::mem::size_of::<Self>())
        }
    }
}

impl TimeSpec {
    /// turn a TimeSpec into a ms value
    pub fn into_ms(&self) -> usize {
        self.tv_sec * MSEC_PER_SEC + (self.tv_nsec / (NSEC_PER_SEC / MSEC_PER_SEC))
    }
    /// get a TimeSpec from a ms value
    pub fn from_ms(ms: usize) -> Self {
        Self {
            tv_sec: ms / MSEC_PER_SEC,
            tv_nsec: (ms % MSEC_PER_SEC) * (NSEC_PER_SEC / MSEC_PER_SEC),
        }
    }
     /// check if is valid
    pub fn is_valid(&self) -> bool {
        self.tv_sec as isize >= 0 && self.tv_nsec as isize >= 0 && self.tv_nsec < 1000_000_000 
    }
    pub const ZERO: Self = Self { tv_sec: 0, tv_nsec: 0 };

    // wall time
    pub fn wall_time() -> Self {
        let mut base_time = TimeSpec{
            tv_sec: 1_757_088_000,
            tv_nsec: 0
        };
        let current = get_current_time_ns();
        base_time.tv_nsec += (current % NSEC_PER_SEC);
        base_time.tv_sec += (current / NSEC_PER_SEC) as usize;
        base_time
    }

     /// to bytes
    pub fn _as_bytes(time: &Self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts((time as *const Self) as *const u8, core::mem::size_of::<Self>())
        }
    }

}

impl From<Duration> for TimeSpec {
    fn from(value: Duration) -> Self {
        Self {
            tv_sec: value.as_secs() as usize,
            tv_nsec: value.subsec_nanos() as usize,
        }
    }
}

impl Into<Duration> for TimeSpec {
    fn into(self) -> Duration {
        Duration::new(self.tv_sec as u64, self.tv_nsec as u32)
    }
}

/// times struct for syscall
#[derive(Clone, Copy)]
#[repr(C)]
pub struct Tms {
    /// user time
    pub utime: usize,
    /// system time
    pub stime: usize,
    /// user cpu of all
    pub cutime: usize,
    /// system cpu of all
    pub cstime: usize,
}

impl Tms {
    /// new a Tms
    pub fn new() -> Self {
        Self {
            utime: 0,
            stime: 0,
            cutime: 0,
            cstime: 0,
        }
    }
    /// new from a TimeRecorder
    pub fn from_time_recorder(time_recorder: &super::recoder::TimeRecorder) -> Self {
        Self {
            utime: time_recorder.user_time().as_micros() as usize,
            stime: time_recorder.kernel_time().as_micros() as usize,
            cutime: time_recorder.child_time_pair().0.as_micros() as usize,
            cstime: time_recorder.child_time_pair().1.as_micros() as usize,
        }
    }
}