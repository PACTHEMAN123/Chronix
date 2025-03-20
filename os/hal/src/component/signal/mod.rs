//! Hardware abstract layer for signal context switching

use super::trap::TrapContext;

pub trait UContextHal {
    /// save current signal context
    /// include: blocked signals, current user_x
    fn save_current_context(old_blocked_sigs: usize, cx: &TrapContext) -> Self;
    /// restore to old trap context using the ucontext
    fn restore_old_context(&self, cx: &mut TrapContext);
}

#[cfg(target_arch = "riscv64")]
mod riscv64;

#[cfg(target_arch = "riscv64")]
#[allow(unused)]
pub use riscv64::*;

#[cfg(target_arch = "loongarch64")]
mod loongarch64;

#[cfg(target_arch = "loongarch64")]
#[allow(unused)]
pub use loongarch64::*;