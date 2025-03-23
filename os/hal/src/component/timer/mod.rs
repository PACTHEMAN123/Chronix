//! Timer Hardware abstraction layer

pub struct Timer;

pub trait TimerHal {
    /// get current time
    fn read() -> usize;
    /// set next time interrupt
    fn set_timer(timer: usize);
    /// get timer freq
    fn get_timer_freq() -> usize;
}

#[cfg(target_arch = "riscv64")]
mod riscv64;

#[cfg(target_arch = "riscv64")]
#[allow(unused)]
pub use riscv64::*;

#[cfg(target_arch = "loongarch64")]
mod loongarch64;

#[cfg(target_arch = "loongarch64")]
pub use loongarch64::*;