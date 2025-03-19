pub trait InstructionHal {
    unsafe fn tlb_flush_addr(vaddr: usize);
    unsafe fn tlb_flush_all();
    unsafe fn enable_interrupt();
    unsafe fn disable_interrupt();
    unsafe fn enable_timer_interrupt();
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
