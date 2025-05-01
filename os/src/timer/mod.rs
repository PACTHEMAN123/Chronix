//! RISC-V timer-related functionality

/// FFI for timer
pub mod ffi;
/// Time recoder for events in tasks and kernel functions
pub mod recoder; 
use hal::{println, timer::{Timer, TimerHal}};
/// timer struct
pub mod timer;
/// time-limited task wrapper
pub mod timed_task;
pub mod clock;
use core::time::Duration;

const TICKS_PER_SEC: usize = 100;
const MSEC_PER_SEC: usize = 1_000;
const USEC_PER_SEC: usize = 1_000_000;
const NSEC_PER_SEC: usize = 1_000_000_000;

/// get current time
pub fn get_current_time() -> usize {
    Timer::read()
}

/// get current time in seconds
pub fn get_current_time_sec() -> usize {
    let cycles = Timer::read() as u128;
    let freq = Timer::get_timer_freq() as u128;
    (cycles / freq) as usize
}

/// get current time in milliseconds
pub fn get_current_time_ms() -> usize {
    let cycles = Timer::read() as u128;
    let freq = Timer::get_timer_freq() as u128;
    ((cycles * MSEC_PER_SEC as u128) / freq) as usize
}

/// get current time in microseconds
pub fn get_current_time_us() -> usize {
    let cycles = Timer::read() as u128;
    let freq = Timer::get_timer_freq() as u128;
    ((cycles * USEC_PER_SEC as u128) / freq) as usize
}
/// get current time in nanoseconds
pub fn get_current_time_ns() -> usize {
    let cycles = Timer::read() as u128;
    let freq = Timer::get_timer_freq() as u128;
    ((cycles * NSEC_PER_SEC as u128) / freq) as usize
}

/// get current time in duration
pub fn get_current_time_duration() -> Duration {
    let mut cycles = Timer::read() as u64;
    let freq = Timer::get_timer_freq() as u64;
    let secs = cycles / freq;
    cycles = cycles % freq;
    let nanos =((cycles as u128 * NSEC_PER_SEC as u128) / freq as u128) as u32;
    Duration::new(secs, nanos)
}

/// set the next timer interrupt
pub fn set_next_trigger() {
    Timer::set_timer(get_current_time() + Timer::get_timer_freq() / TICKS_PER_SEC);
}
