use core::time::Duration;

use hal::println;

use crate::timer::get_current_time_duration;

pub struct TimerGuard<'a> {
    pub name: &'a str,
    pub start: Duration
}

impl<'a> TimerGuard<'a> {
    pub fn new(name: &'a str) -> Self {
        Self { name, start: get_current_time_duration() }
    }
}

impl<'a> Drop for TimerGuard<'a> {
    fn drop(&mut self) {
        println!("{} {:?}", self.name, get_current_time_duration() - self.start);
    }
}
