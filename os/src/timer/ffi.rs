use alloc::collections::btree_map::Values;
use super::get_current_time_ms;
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
}

impl TimeSpec {
    /// turn a TimeSpec into a ms value
    pub fn into_ms(&self) -> usize {
        self.tv_sec * MSEC_PER_SEC + self.tv_nsec / USEC_PER_SEC
    }
    /// get a TimeSpec from a ms value
    pub fn from_ms(ms: usize) -> Self {
        Self {
            tv_sec: ms / MSEC_PER_SEC,
            tv_nsec: (ms % MSEC_PER_SEC) * USEC_PER_SEC,
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