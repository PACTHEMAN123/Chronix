use loongArch64::register::{self, ecfg::LineBasedInterrupt};

use crate::{println, trap::FP_REG_DIRTY};

const POWEROFF_REG_MMIO: usize = 0x8000_0000_100e_001c;
const POWEROFF_VALUE: u8 = 0x34;


use super::{Instruction, InstructionHal};

impl InstructionHal for Instruction {
    unsafe fn tlb_flush_addr(vaddr: usize) {
        core::arch::asm!(
            r"
            dbar 0
            invtlb 0x5, $zero, {0}
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
        unsafe { 
            Instruction::disable_interrupt();
        }
        println!("shutdown, failure: {}", failure);
        unsafe {
            (POWEROFF_REG_MMIO as *mut u8).write_volatile(POWEROFF_VALUE);
        };
        loop {}
    }
    
    fn hart_start(hartid: usize, start_addr: usize, _opaque: usize) {
        loongArch64::ipi::csr_mail_send(start_addr as u64 | 0x9000_0000_0000_0000, hartid, 0);
        loongArch64::ipi::send_ipi_single(hartid, 1);
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
        let cpuid = register::cpuid::read().core_id();
        #[allow(static_mut_refs)]
        unsafe {
            if FP_REG_DIRTY[cpuid] {
                FP_REG_DIRTY[cpuid] = false;
                register::euen::set_fpe(true);
            } else {
                register::euen::set_fpe(false);
            }
        }
    }

    unsafe fn enable_external_interrupt() {
        register::ecfg::set_lie(
            LineBasedInterrupt::HWI0 | LineBasedInterrupt::HWI1 |
            LineBasedInterrupt::HWI2 | LineBasedInterrupt::HWI3 |
            LineBasedInterrupt::HWI4 | LineBasedInterrupt::HWI5 |
            LineBasedInterrupt::HWI6 | LineBasedInterrupt::HWI7 
        );
    }
}