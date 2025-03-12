// Declare the _main_for_arch exists.

use super::constant::Constant;
unsafe extern "Rust" {
    pub(crate) unsafe fn _main_for_arch(id: usize);
}

/// Boot Stack Size.
/// TODO: reduce the boot stack size. Map stack in boot step.
pub const BOOT_STACK_SIZE: usize = Constant::KERNEL_STACK_SIZE;
pub const MAX_PROCESSORS: usize = crate::board::MAX_PROCESSORS;

/// Boot Stack. Boot Stack Size is [STACK_SIZE]
#[unsafe(link_section = ".bss.stack")]
pub(crate) static mut BOOT_STACK: [u8; MAX_PROCESSORS * BOOT_STACK_SIZE] = [0; MAX_PROCESSORS * BOOT_STACK_SIZE];

/// clear BSS segment
fn clear_bss() {
    unsafe extern "C" {
        fn sbss();
        fn ebss();
    }
    unsafe {
        core::slice::from_raw_parts_mut(sbss as usize as *mut u8, ebss as usize - sbss as usize)
            .fill(0);
    }
}

#[macro_export]
macro_rules! define_entry {
    ($main_fn: ident) => {
        #[unsafe(export_name = "_main_for_arch")]
        fn hal_defined_main(id: usize) {
            $main_fn(id);
        }
    };
}

#[cfg(target_arch = "riscv64")]
mod riscv64;

#[cfg(target_arch = "riscv64")]
pub use riscv64::*;

#[cfg(target_arch = "loongarch64")]
mod loongarch64;

#[cfg(target_arch = "loongarch64")]
pub use loongarch64::*;