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

    unsafe fn sie() -> bool {
        sstatus::read().sie()
    }

    unsafe fn enable_timer_interrupt() {
        sie::set_stimer();
    }

    unsafe fn clear_sum() {
        sstatus::clear_sum();
    }

    unsafe fn set_sum() {
        sstatus::set_sum();
    }

    fn shutdown(failure: bool) -> !{
        use sbi_rt::{system_reset, NoReason, Shutdown, SystemFailure};
        if !failure {
            system_reset(Shutdown, NoReason);
        } else {
            system_reset(Shutdown, SystemFailure);
        }
        unreachable!()
    }

    fn hart_start(hartid: usize, start_addr: usize, opaque: usize) {
        sbi_rt::hart_start(hartid, start_addr, opaque);
    }
}