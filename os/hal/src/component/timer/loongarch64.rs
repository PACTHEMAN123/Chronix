use loongArch64::register;

use super::{TimerHal, Timer};

impl TimerHal for Timer {
    fn read() -> usize {
        register::tval::read().raw()
    }
    fn set_timer(timer: usize) {
        register::tcfg::set_init_val(timer);
    }
}