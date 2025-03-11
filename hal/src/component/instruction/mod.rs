pub trait InstructionHal {
    unsafe fn tlb_flush_addr(vaddr: usize);
    unsafe fn tlb_flush_all();
    unsafe fn enable_interrupt();
    unsafe fn disable_interrupt();
}

pub struct Instruction;

#[cfg(target_arch = "riscv64")]
mod riscv64;

#[cfg(target_arch = "riscv64")]
pub use riscv64::*;

#[cfg(target_arch = "loongarch64")]
mod loongarch64;

#[cfg(target_arch = "loongarch64")]
pub use loongarch64::*;
