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

