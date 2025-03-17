//! RISC-V timer-related functionality

/// FFI for timer
pub mod ffi;
/// Time recoder for events in tasks and kernel functions
pub mod recoder; 
use crate::config::CLOCK_FREQ;
use crate::sbi::set_timer;
use riscv::register::time;

use core::time::Duration;

const TICKS_PER_SEC: usize = 100;
const MSEC_PER_SEC: usize = 1000;
const USEC_PER_SEC: usize = 1000000;

/// get current time
pub fn get_current_time() -> usize {
    time::read()
}

/// get current time in milliseconds
pub fn get_current_time_ms() -> usize {
    time::read() / (CLOCK_FREQ / MSEC_PER_SEC)
}

/// get current time in microseconds
pub fn get_current_time_us() -> usize {
    time::read() / (CLOCK_FREQ / USEC_PER_SEC)
}

/// get current time in duration
pub fn get_current_time_duration() -> Duration {
    Duration::from_micros(get_current_time_us() as u64)
}

/// set the next timer interrupt
pub fn set_next_trigger() {
    set_timer(get_current_time() + CLOCK_FREQ / TICKS_PER_SEC);
}
