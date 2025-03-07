use super::get_current_time_ms;


#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
/// TimeVal struct for syscall
pub struct TimeVal {
    /// seconds
    pub sec: usize,
    /// microseconds
    pub usec: usize,
}


#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
/// TimeSpec struct
pub struct TimeSpec {
    /// sec
    pub tv_sec: usize,
    /// nano sec
    pub tv_nsec: usize,
}

