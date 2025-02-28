//! The panic handler

use crate::sbi::shutdown;
use core::{arch::asm, panic::PanicInfo};
use log::*;
use super::mm::VirtAddr;

#[allow(unused)]
fn backtrace() {
    info!("traceback");
    let mut fp: usize;
    unsafe { asm!("mv {}, fp", out(reg)(fp)); }
    while fp != (VirtAddr(fp).floor().0 << 12) {
        fp = unsafe {
            *((fp - 8) as *mut usize)
        };
        info!("{:#x}", unsafe {
            *((fp - 4) as *mut usize)
        });
    }
}

#[panic_handler]
/// panic handler
fn panic(info: &PanicInfo) -> ! {
    // backtrace();
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
    shutdown(true)
}
