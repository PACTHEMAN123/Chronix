use loongArch64::register::{self, ecfg::LineBasedInterrupt};

use super::{Instruction, InstructionHal};

impl InstructionHal for Instruction {
    unsafe fn tlb_flush_addr(vaddr: usize) {
        core::arch::asm!(
            r"
            dbar 0
            invtlb 0x5, $r0, {0}
            ", 
            in(reg) vaddr, 
            options(nostack)
        );
    }

    unsafe fn tlb_flush_all() {
        core::arch::asm!(
            r"
            dbar 0
            invtlb 0x0, $r0, $r0
            ",
            options(nostack)
        );
    }

    unsafe fn enable_interrupt() {
        register::crmd::set_ie(true);
    }

    unsafe fn disable_interrupt() {
        register::crmd::set_ie(false);
    }

    unsafe fn enable_timer_interrupt() {
        register::ecfg::set_lie(LineBasedInterrupt::TIMER);
    }
    
    unsafe fn is_interrupt_enabled() -> bool {
        register::crmd::read().ie()
    }
    
    unsafe fn clear_sum() {
        // do nothing
    }
    
    unsafe fn set_sum() {
        // do nothing
    }
    
    fn shutdown(failure: bool) -> ! {
        log::warn!("shutdown not implemented on loongarch64");
        loop {}
    }
    
    fn hart_start(hartid: usize, start_addr: usize, opaque: usize) {
        panic!("hart_start not implemented on loongarch64")
    }
    
    fn set_tp(processor_addr: usize) {
        unsafe {
            core::arch::asm!(
                "move $tp, {}",
                in(reg) processor_addr,
            );
        }
    }
    
    fn get_tp() -> usize {
        let mut tp: usize;
        unsafe {
            core::arch::asm!(
                "move {}, $tp",
                out(reg) tp,
            );
        }
        tp
    }
    
    fn set_float_status_clean() {
        // do nothing
    }
}