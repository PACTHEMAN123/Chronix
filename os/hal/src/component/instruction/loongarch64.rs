use super::{Instruction, InstructionHal};

impl InstructionHal for Instruction {
    unsafe fn tlb_flush_addr(vaddr: usize) {
        todo!()
    }

    unsafe fn tlb_flush_all() {
        todo!()
    }

    unsafe fn enable_interrupt() {
        todo!()
    }

    unsafe fn disable_interrupt() {
        todo!()
    }

    unsafe fn enable_timer_interrupt() {
        todo!()
    }
}