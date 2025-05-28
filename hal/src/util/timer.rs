use core::time::Duration;

use crate::{println, timer::{Timer, TimerHal}};


const NSEC_PER_SEC: u64 = 1_000_000_000;

/// get current time in duration
pub fn get_current_time_duration() -> Duration {
    let mut cycles = Timer::read() as u64;
    let freq = Timer::get_timer_freq() as u64;
    let secs = cycles / freq;
    cycles = cycles % freq;
    let nanos =((cycles as u128 * NSEC_PER_SEC as u128) / freq as u128) as u32;
    Duration::new(secs, nanos)
}

pub(crate) struct TimerGuard<'a> {
    pub(crate) name: &'a str,
    pub(crate) start: Duration
}

impl<'a> TimerGuard<'a> {
    pub(crate) fn new(name: &'a str) -> Self {
        Self { name, start: get_current_time_duration() }
    }
}

impl<'a> Drop for TimerGuard<'a> {
    fn drop(&mut self) {
        println!("{} {:?}", self.name, get_current_time_duration() - self.start);
    }
}
