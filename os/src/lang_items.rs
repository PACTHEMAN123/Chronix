//! The panic handler

use hal::instruction::{Instruction, InstructionHal};
use core::{arch::asm, panic::PanicInfo};
use hal::{addr::VirtAddrHal, constant::{Constant, ConstantsHal}, println};
use log::*;
use hal::addr::VirtAddr;

#[panic_handler]
/// panic handler
fn panic(info: &PanicInfo) -> ! {
    hal::util::backtrace();
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
