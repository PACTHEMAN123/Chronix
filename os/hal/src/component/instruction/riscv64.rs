use core::arch::asm;

use riscv::register::{sie, sstatus};

use super::{Instruction, InstructionHal};

impl InstructionHal for Instruction {
    unsafe fn tlb_flush_addr(vaddr: usize) {
        asm!("sfence.vma {}, x0", in(reg) vaddr, options(nostack))
    }

    unsafe fn tlb_flush_all() {
        asm!("sfence.vma")
    }

    unsafe fn enable_interrupt() {
        sstatus::set_sie();
    }

    unsafe fn disable_interrupt() {
        sstatus::clear_sie();
    }

    unsafe fn enable_timer_interrupt() {
        sie::set_stimer();
    }
}