//! The panic handler

use hal::instruction::{Instruction, InstructionHal};
use core::{arch::asm, panic::PanicInfo};
use hal::{addr::VirtAddrHal, constant::{Constant, ConstantsHal}, println};
use log::*;
use hal::addr::VirtAddr;

#[allow(unused)]
fn backtrace() {
    println!("traceback: ");
    let mut fp: usize;
    #[cfg(target_arch="riscv64")]
    unsafe { asm!("mv {}, fp", out(reg)(fp)); }
    #[cfg(target_arch="loongarch64")]
    unsafe { asm!("move {}, $fp", out(reg)(fp)); }
    while fp % Constant::PAGE_SIZE != 0 {
        fp = unsafe {
            *((fp - 16) as *mut usize)
        };
        println!("{:#x}", unsafe {
            *((fp - 8) as *mut usize)
        });
    }
}

#[panic_handler]
/// panic handler
fn panic(info: &PanicInfo) -> ! {
    backtrace();
    if let Some(location) = info.location() {
        error!(
            "[kernel] Panicked at {}:{} {}",
            location.file(),
            location.line(),
            info.message()
        );
    } else {
        error!("[kernel] Panicked: {}", info.message());
    }
    Instruction::shutdown(true)
}
