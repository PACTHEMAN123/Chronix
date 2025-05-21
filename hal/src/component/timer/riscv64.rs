//! riscv implementation for timer HAL

use super::{TimerHal, Timer};

use riscv::register::time;

impl TimerHal for Timer {
    fn read() -> usize {
        time::read()
    }
    fn set_timer(timer: usize) {
        sbi_rt::set_timer(timer as _);
    }
    fn get_timer_freq() -> usize {
        return 10000000;
    }
}