use riscv::register::sstatus;

use crate::hal::instruction::{Instruction, InstructionHal};

impl InstructionHal for Instruction {
    unsafe fn tlb_flush_addr(va: usize) {
        core::arch::asm!("sfence.vma {}, x0", in(reg) va, options(nostack));
    }

    unsafe fn tlb_flush_all() {
        core::arch::asm!("sfence.vma");
    }

    unsafe fn disable_interrupt() {
        sstatus::clear_sie();
    }

    unsafe fn enable_interrupt() {
        sstatus::set_sie();
    }
}