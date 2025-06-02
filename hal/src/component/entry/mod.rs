// Declare the _main_for_arch exists.

use core::sync::atomic::AtomicUsize;

use super::constant::{Constant, ConstantsHal};
unsafe extern "Rust" {
    pub(crate) unsafe fn _main_for_arch(id: usize, first: bool) -> bool;
}

/// Boot Stack Size.
/// TODO: reduce the boot stack size. Map stack in boot step.
pub const BOOT_STACK_SIZE: usize = Constant::KERNEL_STACK_SIZE;
pub const MAX_PROCESSORS: usize = crate::board::MAX_PROCESSORS;

/// CINPHAL Is Not Poly Hardware Abstraction Layer
const BANNER: &str = r" ________  ___  ________   ________  ___  ___  ________  ___          
|\   ____\|\  \|\   ___  \|\   __  \|\  \|\  \|\   __  \|\  \         
\ \  \___|\ \  \ \  \\ \  \ \  \|\  \ \  \\\  \ \  \|\  \ \  \        
 \ \  \    \ \  \ \  \\ \  \ \   ____\ \   __  \ \   __  \ \  \       
  \ \  \____\ \  \ \  \\ \  \ \  \___|\ \  \ \  \ \  \ \  \ \  \____  
   \ \_______\ \__\ \__\\ \__\ \__\    \ \__\ \__\ \__\ \__\ \_______\
    \|_______|\|__|\|__| \|__|\|__|     \|__|\|__|\|__|\|__|\|_______|";

/// Boot Stack. Boot Stack Size is [STACK_SIZE]
#[unsafe(link_section = ".bss.stack")]
pub(crate) static mut BOOT_STACK: [u8; MAX_PROCESSORS * BOOT_STACK_SIZE] = [0; MAX_PROCESSORS * BOOT_STACK_SIZE];

#[unsafe(link_section = ".data")] // store in data section, to avoid clear_bss() changing it
pub(crate) static RUNNING_PROCESSOR: AtomicUsize = AtomicUsize::new(0);

/// clear BSS segment
fn clear_bss() {
    unsafe extern "C" {
        fn sbss();
        fn ebss();
    }
    let sbss = sbss as usize;
    let ebss = ebss as usize;
    unsafe {
        let mem = core::slice::from_raw_parts_mut(sbss as *mut u8, ebss as usize - sbss as usize);
        mem.fill(0);
    }
}

#[macro_export]
macro_rules! define_entry {
    ($main_fn: ident) => {
        #[unsafe(export_name = "_main_for_arch")]
        fn hal_defined_main(id: usize, first: bool) -> bool {
            $main_fn(id, first)
        }
    };
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