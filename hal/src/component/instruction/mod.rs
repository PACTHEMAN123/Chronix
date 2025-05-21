pub trait InstructionHal {
    unsafe fn tlb_flush_addr(vaddr: usize);
    unsafe fn tlb_flush_all();
    unsafe fn enable_interrupt();
    unsafe fn disable_interrupt();
    unsafe fn is_interrupt_enabled() -> bool;
    unsafe fn enable_timer_interrupt();
    unsafe fn enable_external_interrupt();
    unsafe fn clear_sum();
    unsafe fn set_sum();
    fn shutdown(failure: bool) -> !;
    fn hart_start(hartid: usize, start_addr: usize, opaque: usize);
    fn set_tp(processor_addr: usize);
    fn get_tp() -> usize;
    fn set_float_status_clean();
}

pub struct Instruction;

#[cfg(target_arch = "riscv64")]
mod riscv64;

#[cfg(target_arch = "riscv64")]
#[allow(unused)]
pub use riscv64::*;

#[cfg(target_arch = "loongarch64")]
mod loongarch64;

#[cfg(target_arch = "loongarch64")]
pub use loongarch64::*;
