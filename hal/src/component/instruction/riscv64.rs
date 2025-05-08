use core::arch::asm;

use riscv::register;

use super::{Instruction, InstructionHal};

impl InstructionHal for Instruction {
    unsafe fn tlb_flush_addr(vaddr: usize) {
        riscv::asm::sfence_vma(0, vaddr);
    }

    unsafe fn tlb_flush_all() {
        riscv::asm::sfence_vma_all();
    }

    unsafe fn enable_interrupt() {
        register::sstatus::set_sie();
    }

    unsafe fn disable_interrupt() {
        register::sstatus::clear_sie();
    }

    unsafe fn is_interrupt_enabled() -> bool {
        register::sstatus::read().sie()
    }

    unsafe fn enable_timer_interrupt() {
        register::sie::set_stimer();
    }

    unsafe fn clear_sum() {
        register::sstatus::clear_sum();
    }

    unsafe fn set_sum() {
        register::sstatus::set_sum();
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
    
    #[inline(always)]
    fn set_tp(processor_addr: usize) {
        unsafe {
            asm!(
                "mv tp, {}",
                in(reg) processor_addr,
             )
        }
    }
    
    #[inline(always)]
    fn get_tp() -> usize {
        let tp: usize;
        unsafe {
            asm!(
                "mv {}, tp",
                out(reg) tp,
            );
        }
        tp
    }
    
    fn set_float_status_clean() {
        unsafe {
            register::sstatus::set_fs(register::sstatus::FS::Clean);
        }
    }
}