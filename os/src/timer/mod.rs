//! RISC-V timer-related functionality

/// FFI for timer
pub mod ffi;
/// Time recoder for events in tasks and kernel functions
pub mod recoder; 
use hal::timer::{Timer, TimerHal};
pub mod timer;
/// time-limited task wrapper
pub mod timed_task;
pub mod clock;
use core::time::Duration;

const TICKS_PER_SEC: usize = 100;
const MSEC_PER_SEC: usize = 1000;
const USEC_PER_SEC: usize = 1000000;

/// get current time
pub fn get_current_time() -> usize {
    Timer::read()
}

/// get current time in milliseconds
pub fn get_current_time_ms() -> usize {
    Timer::read() / (Timer::get_timer_freq() / MSEC_PER_SEC)
}

/// get current time in microseconds
pub fn get_current_time_us() -> usize {
    Timer::read() / (Timer::get_timer_freq() / USEC_PER_SEC)
}

/// get current time in duration
pub fn get_current_time_duration() -> Duration {
    Duration::from_micros(get_current_time_us() as u64)
}

/// set the next timer interrupt
pub fn set_next_trigger() {
    Timer::set_timer(get_current_time() + Timer::get_timer_freq() / TICKS_PER_SEC);
}
