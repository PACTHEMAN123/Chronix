use loongArch64::register;

use super::{TimerHal, Timer};

impl TimerHal for Timer {
    fn read() -> usize {
        let mut counter: usize;
        unsafe {
            core::arch::asm!(
            "rdtime.d {},{}",
            out(reg)counter,
            out(reg)_,
            );
        }
        counter
    }
    fn set_timer(timer: usize) {
        register::tcfg::set_init_val(timer);
        register::ticlr::clear_timer_interrupt();
        register::tcfg::set_en(true);
        register::tcfg::set_periodic(true);
    }

    fn get_timer_freq() -> usize {
        loongArch64::time::get_timer_freq()
    }
}